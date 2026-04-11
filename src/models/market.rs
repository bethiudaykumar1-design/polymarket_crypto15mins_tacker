// src/models/market.rs

use std::sync::Arc;
use tokio::sync::Mutex;

const THRESHOLDS: [(f64, &str); 6] = [
    (0.35, "035"), (0.30, "030"), (0.25, "025"),
    (0.20, "020"), (0.15, "015"), (0.10, "010")
];

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
    
    // UP thresholds
    pub up_035: bool, pub up_030: bool, pub up_025: bool,
    pub up_020: bool, pub up_015: bool, pub up_010: bool,
    
    pub up_from_035: f64, pub up_from_030: f64, pub up_from_025: f64,
    pub up_from_020: f64, pub up_from_015: f64, pub up_from_010: f64,
    
    pub up_minutes_035: Option<i32>, pub up_minutes_030: Option<i32>, pub up_minutes_025: Option<i32>,
    pub up_minutes_020: Option<i32>, pub up_minutes_015: Option<i32>, pub up_minutes_010: Option<i32>,
    
    // DOWN thresholds
    pub down_035: bool, pub down_030: bool, pub down_025: bool,
    pub down_020: bool, pub down_015: bool, pub down_010: bool,
    
    pub down_from_035: f64, pub down_from_030: f64, pub down_from_025: f64,
    pub down_from_020: f64, pub down_from_015: f64, pub down_from_010: f64,
    
    pub down_minutes_035: Option<i32>, pub down_minutes_030: Option<i32>, pub down_minutes_025: Option<i32>,
    pub down_minutes_020: Option<i32>, pub down_minutes_015: Option<i32>, pub down_minutes_010: Option<i32>,
    
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
            up_035: false, up_030: false, up_025: false, up_020: false, up_015: false, up_010: false,
            up_from_035: 0.0, up_from_030: 0.0, up_from_025: 0.0, up_from_020: 0.0, up_from_015: 0.0, up_from_010: 0.0,
            up_minutes_035: None, up_minutes_030: None, up_minutes_025: None,
            up_minutes_020: None, up_minutes_015: None, up_minutes_010: None,
            down_035: false, down_030: false, down_025: false, down_020: false, down_015: false, down_010: false,
            down_from_035: 0.0, down_from_030: 0.0, down_from_025: 0.0, down_from_020: 0.0, down_from_015: 0.0, down_from_010: 0.0,
            down_minutes_035: None, down_minutes_030: None, down_minutes_025: None,
            down_minutes_020: None, down_minutes_015: None, down_minutes_010: None,
            result: None,
        }
    }
    
    pub fn update_prices(&mut self, up: f64, down: f64, now_ts: i64) {
        self.last_up_price = up;
        self.last_down_price = down;
        
        self.update_side("UP", up, now_ts);
        self.update_side("DOWN", down, now_ts);
    }
    
    fn update_side(&mut self, side: &str, price: f64, now_ts: i64) {
        for &(threshold, name) in THRESHOLDS.iter() {
            let (dipped, from_price, minutes_field) = match (side, name) {
                ("UP", "035") => (&mut self.up_035, &mut self.up_from_035, &mut self.up_minutes_035),
                ("UP", "030") => (&mut self.up_030, &mut self.up_from_030, &mut self.up_minutes_030),
                ("UP", "025") => (&mut self.up_025, &mut self.up_from_025, &mut self.up_minutes_025),
                ("UP", "020") => (&mut self.up_020, &mut self.up_from_020, &mut self.up_minutes_020),
                ("UP", "015") => (&mut self.up_015, &mut self.up_from_015, &mut self.up_minutes_015),
                ("UP", "010") => (&mut self.up_010, &mut self.up_from_010, &mut self.up_minutes_010),
                ("DOWN", "035") => (&mut self.down_035, &mut self.down_from_035, &mut self.down_minutes_035),
                ("DOWN", "030") => (&mut self.down_030, &mut self.down_from_030, &mut self.down_minutes_030),
                ("DOWN", "025") => (&mut self.down_025, &mut self.down_from_025, &mut self.down_minutes_025),
                ("DOWN", "020") => (&mut self.down_020, &mut self.down_from_020, &mut self.down_minutes_020),
                ("DOWN", "015") => (&mut self.down_015, &mut self.down_from_015, &mut self.down_minutes_015),
                ("DOWN", "010") => (&mut self.down_010, &mut self.down_from_010, &mut self.down_minutes_010),
                _ => continue,
            };
            
            // Check dip
            if price < threshold && !*dipped {
                *dipped = true;
                *from_price = price;
                let minutes_left = (self.end_ts - now_ts) / 60;
                *minutes_field = Some(minutes_left as i32);
                // println!("📉 {}: {} dropped below {:.2} at {:.4}", self.symbol, side, threshold, price);
            }
            
            // Check bounce
            if *dipped && price > *from_price {
                *from_price = price;
                // println!("📈 {}: {} bounced to {:.4} (from {:.2})", self.symbol, side, price, threshold);
            }
        }
    }
}