use std::sync::Arc;
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

mod config;
mod utils;
mod api;
mod models;
mod manager;
mod db;

use manager::MarketManager;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    println!("🚀 Multi-Market Polymarket Watcher (15-min markets)");
    println!("📊 Tracking: BTC, ETH, SOL, XRP");
    
    
    // Initialize database
    let db = match env::var("DATABASE_URL") {
        Ok(database_url) => {
            match db::Database::new(&database_url).await {
                Ok(db) => {
                    println!("✅ Database connected");
                    Some(Arc::new(db))
                }
                Err(e) => {
                    eprintln!("⚠️ Database connection failed: {}", e);
                    eprintln!("⚠️ Continuing without database");
                    None
                }
            }
        }
        Err(_) => {
            println!("⚠️ DATABASE_URL not set, running without database");
            None
        }
    };
    
    // Create and initialize market manager with database
    let mut manager = MarketManager::new(db);
    manager.initialize().await;
    
    
    
    loop {
        manager.process_all().await;
        sleep(Duration::from_secs(2)).await;
    }
}