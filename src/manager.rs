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
    println!("🏁 {}: Market ending", self.config.symbol);
    
    // Get market data before shifting
    let market_info = if let Some(current) = &self.current_market {
        let data = current.lock().await;
        Some((
            data.symbol.clone(),
            data.market_id.clone(),
            data.title.clone(),
            data.start_up_price,
            data.start_down_price,
            data.high_up_price,      // Use high instead of low
            data.high_down_price,    // Use high instead of low
            data.last_up_price,
            data.last_down_price,
        ))
    } else {
        None
    };
    
    // Save to database if available
    if let Some(db) = &self.db {
        if let Some((
            symbol, market_id, title,
            start_up, start_down, high_up, high_down, last_up, last_down
        )) = market_info {
            
            // Determine result
            let result = resolve::resolve_result(
                &market_id,
                last_up,
                last_down,
            ).await;
            
            // println!("📊 {}: Result - {}", symbol, result);
            // println!("📈 {}: High UP: {:.4}, High DOWN: {:.4}", symbol, high_up, high_down);
            
            // Save to database
            match db.save_market_result(
                &symbol,
                &market_id,
                &title,
                start_up,
                start_down,
                high_up,
                high_down,
                last_up,
                last_down,
                &result,
            ).await {
                Ok(_) => println!("💾 {}: Saved to database", symbol),
                Err(e) => eprintln!("❌ {}: Failed to save: {}", symbol, e),
            }
        }
    } else {
        println!("⚠️ {}: No database connection, skipping save", self.config.symbol);
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