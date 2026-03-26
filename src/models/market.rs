use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug,Clone)]
pub struct MarketData {
    
    pub symbol:String,
    pub market_id: String,
    pub slug: String,
    pub title: String,

    // timetracking
    pub start_ts: i64,         
    pub end_ts: i64,           
    pub interval: i64,

    // Token IDs (for WebSocket)
    pub up_token_id: String,
    pub down_token_id: String,

    // price tracking
    pub start_up_price: f64,
    pub start_down_price: f64,
    pub low_up_price: f64,     
    pub low_down_price: f64,   
    pub last_up_price: f64,
    pub last_down_price: f64,

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
    )->Self {
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
            low_up_price: start_up_price,
            low_down_price: start_down_price,
            last_up_price: start_up_price,
            last_down_price: start_down_price,
            result: None,

        }

    }

    // update prices from ws
    pub fn update_prices(&mut self,up:f64,down:f64){
        self.last_up_price=up;
        self.last_down_price=down;

        if up < self.low_up_price {
            self.low_up_price=up;
        }
        if down<self.low_down_price {
            self.low_down_price=down;
        }
    }
}