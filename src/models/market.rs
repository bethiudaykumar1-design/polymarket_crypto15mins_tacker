// src/models/market.rs

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct MarketData {
    // Identification
    pub symbol: String,
    pub market_id: String,
    pub slug: String,
    pub title: String,
    
    // Time tracking
    pub start_ts: i64,
    pub end_ts: i64,
    pub interval: i64,
    
    // Token IDs
    pub up_token_id: String,
    pub down_token_id: String,
    
    // Current prices
    pub last_up_price: f64,
    pub last_down_price: f64,
    
    // Track 30% bounce (only one side)
    pub side_below_30: Option<String>,  // "UP" or "DOWN"
    pub minutes_left_at_below_30: Option<i32>,
    pub high_from_30: f64,
    
    // Result
    pub result: Option<String>,
}

impl MarketData {
    pub fn new(
        symbol: String,
        market_id: String,
        slug: String,
        title: String,
        start_ts: i64,
        end_ts: i64,
        interval: i64,
        up_token_id: String,
        down_token_id: String,
        start_up_price: f64,
        start_down_price: f64,
    ) -> Self {
        Self {
            symbol,
            market_id,
            slug,
            title,
            start_ts,
            end_ts,
            interval,
            up_token_id,
            down_token_id,
            last_up_price: start_up_price,
            last_down_price: start_down_price,
            side_below_30: None,
            minutes_left_at_below_30: None,
            high_from_30: 0.0,
            result: None,
        }
    }
    
    pub fn update_prices(&mut self, up: f64, down: f64, now_ts: i64) {
        self.last_up_price = up;
        self.last_down_price = down;
        
        // Only track if we haven't already recorded a below-30 event
        if self.side_below_30.is_none() {
            let mut side: Option<String> = None;
            let mut price_at_below: f64 = 0.0;
            
            // Check UP below 30%
            if up < 0.30 {
                side = Some("UP".to_string());
                price_at_below = up;
            }
            // Check DOWN below 30%
            else if down < 0.30 {
                side = Some("DOWN".to_string());
                price_at_below = down;
            }
            
            if let Some(side_str) = side {
                let minutes_left = (self.end_ts - now_ts) / 60;
                self.side_below_30 = Some(side_str.clone());
                self.minutes_left_at_below_30 = Some(minutes_left as i32);
                self.high_from_30 = price_at_below;
                
                // println!("📉 {}: {} dropped below 30% at {:.4}", 
                //     self.symbol, side_str, price_at_below);
                // println!("⏰ {}: {} minutes left in market", self.symbol, minutes_left);
            }
        }
        
        // Track bounce if we have a recorded side
        if let Some(ref side) = self.side_below_30 {
            let current_price = if side == "UP" { up } else { down };
            
            if current_price > self.high_from_30 {
                self.high_from_30 = current_price;
                // println!("📈 {}: {} bounced to {:.4}", self.symbol, side, current_price);
            }
        }
    }
}