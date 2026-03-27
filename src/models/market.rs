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
    
    // Price tracking
    pub start_up_price: f64,
    pub start_down_price: f64,
    pub high_up_price: f64,      // Replace low with high
    pub high_down_price: f64,    // Replace low with high
    pub last_up_price: f64,
    pub last_down_price: f64,
    
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
            start_up_price,
            start_down_price,
            high_up_price: start_up_price,   // Track highest
            high_down_price: start_down_price, // Track highest
            last_up_price: start_up_price,
            last_down_price: start_down_price,
            result: None,
        }
    }
    
    pub fn update_prices(&mut self, up: f64, down: f64) {
        self.last_up_price = up;
        self.last_down_price = down;
        
        // Track highs (replace lows)
        if up > self.high_up_price {
            self.high_up_price = up;
        }
        
        if down > self.high_down_price {
            self.high_down_price = down;
        }
    }
}