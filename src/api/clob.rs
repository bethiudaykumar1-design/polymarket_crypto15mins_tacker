// src/api/clob.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MidpointResponse {
    mid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LastTradeResponse {
    price: Option<String>,
}

/// Fetch midpoint price from CLOB API
/// Midpoint is the average of best bid and ask
pub async fn fetch_midpoint(token_id: &str) -> Option<f64> {
    let url = format!(
        "https://clob.polymarket.com/midpoint?token_id={}",
        token_id
    );
    
    let client = reqwest::Client::new();
    
    match client.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<MidpointResponse>().await {
                    Ok(data) => {
                        if let Some(mid_str) = data.mid {
                            match mid_str.parse::<f64>() {
                                Ok(price) => return Some(price),
                                Err(e) => println!("⚠️ Failed to parse midpoint price: {}", e),
                            }
                        }
                    }
                    Err(e) => println!("⚠️ Failed to parse midpoint response: {}", e),
                }
            }
            None
        }
        Err(e) => {
            println!("⚠️ Failed to fetch midpoint for {}: {}", token_id, e);
            None
        }
    }
}

/// Fetch last trade price as fallback
pub async fn fetch_last_trade(token_id: &str) -> Option<f64> {
    let url = format!(
        "https://clob.polymarket.com/last-trade?token_id={}",
        token_id
    );
    
    let client = reqwest::Client::new();
    
    match client.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<LastTradeResponse>().await {
                    Ok(data) => {
                        if let Some(price_str) = data.price {
                            match price_str.parse::<f64>() {
                                Ok(price) => return Some(price),
                                Err(e) => println!("⚠️ Failed to parse last trade price: {}", e),
                            }
                        }
                    }
                    Err(e) => println!("⚠️ Failed to parse last trade response: {}", e),
                }
            }
            None
        }
        Err(e) => {
            println!("⚠️ Failed to fetch last trade for {}: {}", token_id, e);
            None
        }
    }
}

/// Fetch initial price with fallback strategy:
/// 1. Try midpoint first (most accurate)
/// 2. If midpoint fails, try last trade
/// 3. If both fail, use default 0.5
pub async fn fetch_initial_price(token_id: &str) -> f64 {
    // Try midpoint first
    if let Some(price) = fetch_midpoint(token_id).await {
        // println!("  ✅ Got midpoint price: {:.4}", price);
        return price;
    }
    
    // Fallback to last trade
    if let Some(price) = fetch_last_trade(token_id).await {
        // println!("  ✅ Got last trade price: {:.4}", price);
        return price;
    }
    
    // Default if both fail
    // println!("  ⚠️ Could not fetch price, using default 0.5");
    0.5
}