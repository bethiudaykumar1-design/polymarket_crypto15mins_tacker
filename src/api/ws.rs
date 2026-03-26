// src/api/ws.rs

use std::collections::HashMap;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use url::Url;

use crate::models::market::MarketData;
use crate::utils::time;

pub async fn start_ws(
    up_id: String,
    down_id: String,
    market_data: Arc<Mutex<MarketData>>,
    expected_market_id: String,
    symbol: String,  // For logging which market this is
) {
    // println!("🔌 {}: Connecting WebSocket...", symbol);
    
    let url = Url::parse("wss://ws-subscriptions-clob.polymarket.com/ws/market")
        .expect("Invalid WebSocket URL");
    
    match connect_async(url).await {
        Ok((mut ws, _)) => {
            // println!("✅ {}: WebSocket connected", symbol);
            
            // Subscribe to both UP and DOWN tokens
            let subscription = json!({
                "assets_ids": [up_id, down_id],
                "type": "market"
            });
            
            if let Err(e) = ws.send(
                tokio_tungstenite::tungstenite::Message::Text(subscription.to_string()),
            ).await {
                println!("❌ {}: Failed to subscribe: {}", symbol, e);
                return;
            }
            
            // println!("📡 {}: Subscribed to price updates", symbol);
            
            // Store latest prices
            let mut prices: HashMap<String, f64> = HashMap::new();
            
            // Listen for price updates
            while let Some(msg) = ws.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        println!("⚠️ {}: WebSocket error: {}", symbol, e);
                        break;
                    }
                };
                
                if let tokio_tungstenite::tungstenite::Message::Text(txt) = msg {
                    // Parse the JSON message
                    let ws_data: serde_json::Value = match serde_json::from_str(&txt) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    
                    // We only care about last_trade_price events
                    if ws_data["event_type"] == "last_trade_price" {
                        let asset_id = match ws_data["asset_id"].as_str() {
                            Some(a) => a,
                            None => continue,
                        };
                        
                        let price = match ws_data["price"].as_str() {
                            Some(p) => p.parse::<f64>().ok(),
                            None => continue,
                        };
                        
                        if let Some(price) = price {
                            // Store the latest price for this asset
                            prices.insert(asset_id.to_string(), price);
                            
                            // Check if we have both UP and DOWN prices
                            if let (Some(up_price), Some(down_price)) = 
                                (prices.get(&up_id), prices.get(&down_id)) 
                            {
                                // Check if market is still active
                                let should_continue = {
                                    let data = market_data.lock().await;
                                    let now = time::now_ts();
                                    
                                    // Stop if market has expired (ended + 5 seconds buffer)
                                    if now > data.end_ts + 5 {
                                        // println!("🛑 {}: Market expired, stopping WebSocket", symbol);
                                        false
                                    }
                                    // Stop if market ID changed (shouldn't happen, but safety check)
                                    else if data.market_id != expected_market_id {
                                        // println!("🛑 {}: Market changed, stopping WebSocket", symbol);
                                        false
                                    }
                                    else {
                                        true
                                    }
                                };
                                
                                if !should_continue {
                                    return;
                                }
                                
                               
                            
                                    // println!("{},{},->{}",up_price,down_price,symbol);
                                

                                // Update market data with new prices
                                let mut data = market_data.lock().await;
                                data.update_prices(*up_price, *down_price);
                                
                                // Optional: Print every 10th update to avoid spam
                                // You can uncomment for debugging
                                // static mut COUNTER: u32 = 0;
                                // unsafe {
                                //     COUNTER += 1;
                                //     if COUNTER % 10 == 0 {
                                //         println!("📊 {}: UP: {:.4}, DOWN: {:.4}", 
                                //                  symbol, up_price, down_price);
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
            
            // println!("⚠️ {}: WebSocket disconnected", symbol);
        }
        Err(e) => {
            println!("❌ {}: Failed to connect WebSocket: {:?}", symbol, e);
            // Brief sleep before returning
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    // println!("🏁 {}: WebSocket task ending", symbol);
}