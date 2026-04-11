// src/manager.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::MarketConfig;
use crate::models::market::MarketData;
use crate::api::{gamma, clob, ws, resolve};
use crate::utils::time;
use crate::db::Database;

pub struct MarketManager {
    handlers: HashMap<String, MarketHandler>,
    db: Option<Arc<Database>>,  // Add database
}

pub struct MarketHandler {
    config: MarketConfig,
    current_market: Option<Arc<Mutex<MarketData>>>,
    next_market: Option<Arc<Mutex<MarketData>>>,
    ws_handle: Option<JoinHandle<()>>,
    db: Option<Arc<Database>>,  // Add database
}

impl MarketManager {
    pub fn new(db: Option<Arc<Database>>) -> Self {
        Self {
            handlers: HashMap::new(),
            db,
        }
    }
    
    pub async fn initialize(&mut self) {
        let all_markets = crate::config::get_all_markets();
        
        for config in all_markets {
            // println!("📊 Initializing handler for {} market", config.symbol);
            
            let handler = MarketHandler::new(config, self.db.clone());  // Pass db clone
            self.handlers.insert(handler.config.symbol.clone(), handler);
        }
        
        // println!("✅ All market handlers initialized");
    }
    
    pub async fn process_all(&mut self) {
        for (_, handler) in self.handlers.iter_mut() {
            handler.process().await;
        }
    }
}

impl MarketHandler {
    pub fn new(config: MarketConfig, db: Option<Arc<Database>>) -> Self {
        Self {
            config,
            current_market: None,
            next_market: None,
            ws_handle: None,
            db,
        }
    }
    
    pub async fn process(&mut self) {
        let now = time::now_ts();
        let window_start = time::current_window_ts(now);
        let window_end = window_start + self.config.interval;
        
        if self.current_market.is_none() {
            self.initialize_current_market(window_start, window_end).await;
        }
        
        if let Some(current) = &self.current_market {
            let end_ts = {
                let data = current.lock().await;
                data.end_ts
            };
            
            let secs_left = end_ts - now;
            
            if secs_left <= 20 && self.next_market.is_none() {
                self.preload_next_market(window_end).await;
            }
            
            if secs_left <= 0 {
                self.end_current_market().await;
            }
        }
    }
    
    async fn initialize_current_market(&mut self, start_ts: i64, end_ts: i64) {
        let slug = self.config.generate_slug(start_ts);
        // println!("⏳ {}: Initializing market: {}", self.config.symbol, slug);
        
        if let Some(market) = gamma::fetch_with_retry(&slug).await {
            if let Some((up_token, down_token)) = gamma::extract_token_ids(&market) {
                let up_price = clob::fetch_initial_price(&up_token).await;
                let down_price = clob::fetch_initial_price(&down_token).await;
                
                let market_data = MarketData::new(
                    self.config.symbol.clone(),
                    market.id.clone(),
                    slug,
                    market.question.clone(),
                    start_ts,
                    end_ts,
                    self.config.interval,
                    up_token.clone(),
                    down_token.clone(),
                    up_price,
                    down_price,
                );
                
                let shared = Arc::new(Mutex::new(market_data));
                
                let ws_handle = tokio::spawn(ws::start_ws(
                    up_token,
                    down_token,
                    shared.clone(),
                    market.id,
                    self.config.symbol.clone(),
                ));
                
                self.current_market = Some(shared);
                self.ws_handle = Some(ws_handle);
                
                // println!("✅ {}: Market started - UP: {:.4}, DOWN: {:.4}", 
                    // self.config.symbol, up_price, down_price);
            }
        }
    }
    
    async fn preload_next_market(&mut self, next_start_ts: i64) {
        let next_end_ts = next_start_ts + self.config.interval;
        let slug = self.config.generate_slug(next_start_ts);
        
        // println!("🔮 {}: Preloading next market: {}", self.config.symbol, slug);
        
        if let Some(market) = gamma::fetch_with_retry(&slug).await {
            if let Some((up_token, down_token)) = gamma::extract_token_ids(&market) {
                let up_price = clob::fetch_initial_price(&up_token).await;
                let down_price = clob::fetch_initial_price(&down_token).await;
                
                let market_data = MarketData::new(
                    self.config.symbol.clone(),
                    market.id.clone(),
                    slug,
                    market.question.clone(),
                    next_start_ts,
                    next_end_ts,
                    self.config.interval,
                    up_token.clone(),
                    down_token.clone(),
                    up_price,
                    down_price,
                );
                
                let shared = Arc::new(Mutex::new(market_data));
                
                tokio::spawn(ws::start_ws(
                    up_token,
                    down_token,
                    shared.clone(),
                    market.id,
                    self.config.symbol.clone(),
                ));
                
                self.next_market = Some(shared);
                // println!("✅ {}: Next market preloaded", self.config.symbol);
            }
        }
    }
    
   async fn end_current_market(&mut self) {
    // println!("🏁 {}: Market ending", self.config.symbol);
    
    // Get market data before shifting
    let market_info = if let Some(current) = &self.current_market {
        let data = current.lock().await;
        Some((
            data.symbol.clone(),
            data.market_id.clone(),
            data.title.clone(),
            data.last_up_price,
            data.last_down_price,
            // UP data
            data.up_035, data.up_030, data.up_025, data.up_020, data.up_015, data.up_010,
            data.up_from_035, data.up_from_030, data.up_from_025, data.up_from_020, data.up_from_015, data.up_from_010,
            data.up_minutes_035, data.up_minutes_030, data.up_minutes_025,
            data.up_minutes_020, data.up_minutes_015, data.up_minutes_010,
            // DOWN data
            data.down_035, data.down_030, data.down_025, data.down_020, data.down_015, data.down_010,
            data.down_from_035, data.down_from_030, data.down_from_025, data.down_from_020, data.down_from_015, data.down_from_010,
            data.down_minutes_035, data.down_minutes_030, data.down_minutes_025,
            data.down_minutes_020, data.down_minutes_015, data.down_minutes_010,
        ))
    } else {
        None
    };
    
    // Save to database if available
    if let Some(db) = &self.db {
        if let Some((
            symbol, market_id, title, last_up, last_down,
            up_035, up_030, up_025, up_020, up_015, up_010,
            up_from_035, up_from_030, up_from_025, up_from_020, up_from_015, up_from_010,
            up_minutes_035, up_minutes_030, up_minutes_025, up_minutes_020, up_minutes_015, up_minutes_010,
            down_035, down_030, down_025, down_020, down_015, down_010,
            down_from_035, down_from_030, down_from_025, down_from_020, down_from_015, down_from_010,
            down_minutes_035, down_minutes_030, down_minutes_025, down_minutes_020, down_minutes_015, down_minutes_010,
        )) = market_info {
            
            // Determine result
            let result = resolve::resolve_result(&market_id, last_up, last_down).await;
            
            // println!("📊 {}: Result - {}", symbol, result);
            
            // Print UP dips summary
            // if up_035 { println!("📈 {}: UP dipped below 0.35, bounced to {:.4}", symbol, up_from_035); }
            // if up_030 { println!("📈 {}: UP dipped below 0.30, bounced to {:.4}", symbol, up_from_030); }
            // if up_025 { println!("📈 {}: UP dipped below 0.25, bounced to {:.4}", symbol, up_from_025); }
            // if up_020 { println!("📈 {}: UP dipped below 0.20, bounced to {:.4}", symbol, up_from_020); }
            // if up_015 { println!("📈 {}: UP dipped below 0.15, bounced to {:.4}", symbol, up_from_015); }
            // if up_010 { println!("📈 {}: UP dipped below 0.10, bounced to {:.4}", symbol, up_from_010); }
            
            // Print DOWN dips summary
            // if down_035 { println!("📉 {}: DOWN dipped below 0.35, bounced to {:.4}", symbol, down_from_035); }
            // if down_030 { println!("📉 {}: DOWN dipped below 0.30, bounced to {:.4}", symbol, down_from_030); }
            // if down_025 { println!("📉 {}: DOWN dipped below 0.25, bounced to {:.4}", symbol, down_from_025); }
            // if down_020 { println!("📉 {}: DOWN dipped below 0.20, bounced to {:.4}", symbol, down_from_020); }
            // if down_015 { println!("📉 {}: DOWN dipped below 0.15, bounced to {:.4}", symbol, down_from_015); }
            // if down_010 { println!("📉 {}: DOWN dipped below 0.10, bounced to {:.4}", symbol, down_from_010); }
            
            // Save to database
            match db.save_market_result(
                &symbol, &market_id, &title, last_up, last_down, &result,
                up_035, up_030, up_025, up_020, up_015, up_010,
                up_from_035, up_from_030, up_from_025, up_from_020, up_from_015, up_from_010,
                up_minutes_035, up_minutes_030, up_minutes_025, up_minutes_020, up_minutes_015, up_minutes_010,
                down_035, down_030, down_025, down_020, down_015, down_010,
                down_from_035, down_from_030, down_from_025, down_from_020, down_from_015, down_from_010,
                down_minutes_035, down_minutes_030, down_minutes_025, down_minutes_020, down_minutes_015, down_minutes_010,
            ).await {
                Ok(_) => {},
                Err(e) => eprintln!("❌ {}: Failed to save: {}", symbol, e),
            }
        }
    }
    
    self.shift_markets();
}

    
    fn shift_markets(&mut self) {
        self.current_market = self.next_market.take();
        self.next_market = None;
        self.ws_handle = None;
        // println!("🔄 {}: Markets shifted", self.config.symbol);
    }
}