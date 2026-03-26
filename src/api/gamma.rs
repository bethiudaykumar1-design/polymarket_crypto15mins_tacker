// src/api/gamma.rs

use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Clone)]
pub struct Market {
    pub id: String,
    pub question: String,
    #[serde(rename = "clobTokenIds")]
    pub clob_token_ids: String,  // JSON string of token IDs
    pub outcomes: String,         // JSON string of outcomes ["Up", "Down"]
    pub active: bool,
    pub closed: bool,
}

/// Fetch market by slug from Gamma API
pub async fn fetch_market_by_slug(slug: &str) -> Option<Market> {
    let url = format!(
        "https://gamma-api.polymarket.com/markets?slug={}",
        slug
    );
    
    let client = reqwest::Client::new();
    
    match client.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                let markets: Vec<Market> = match res.json().await {
                    Ok(data) => data,
                    Err(e) => {
                        println!("⚠️ Failed to parse JSON: {}", e);
                        return None;
                    }
                };
                return markets.into_iter().next();
            }
            None
        }
        Err(e) => {
            println!("⚠️ Failed to fetch market {}: {}", slug, e);
            None
        }
    }
}

/// Extract UP and DOWN token IDs from market
pub fn extract_token_ids(market: &Market) -> Option<(String, String)> {
    // Parse the JSON strings
    let tokens: Vec<String> = match serde_json::from_str(&market.clob_token_ids) {
        Ok(t) => t,
        Err(e) => {
            println!("⚠️ Failed to parse token IDs: {}", e);
            return None;
        }
    };
    
    let outcomes: Vec<String> = match serde_json::from_str(&market.outcomes) {
        Ok(o) => o,
        Err(e) => {
            println!("⚠️ Failed to parse outcomes: {}", e);
            return None;
        }
    };
    
    // Check we have exactly 2 tokens and 2 outcomes
    if tokens.len() != 2 || outcomes.len() != 2 {
        println!("⚠️ Expected 2 tokens/outcomes, got {} tokens, {} outcomes", 
                 tokens.len(), outcomes.len());
        return None;
    }
    
    let mut up_token = String::new();
    let mut down_token = String::new();
    
    // Find which token is UP and which is DOWN
    for (i, outcome) in outcomes.iter().enumerate() {
        match outcome.to_lowercase().as_str() {
            "up" => up_token = tokens[i].clone(),
            "down" => down_token = tokens[i].clone(),
            _ => {}
        }
    }
    
    if up_token.is_empty() || down_token.is_empty() {
        println!("⚠️ Could not find both UP and DOWN outcomes");
        return None;
    }
    
    Some((up_token, down_token))
}

/// Fetch market with retry logic (market may not exist yet)
pub async fn fetch_with_retry(slug: &str) -> Option<Market> {
    for attempt in 1..=10 {
        // println!("  Attempt {}/10: Fetching market: {}", attempt, slug);
        
        if let Some(market) = fetch_market_by_slug(slug).await {
            // println!("  ✅ Market found!");
            return Some(market);
        }
        
        println!("  ⏳ Market not available yet, retrying in 3 seconds...");
        sleep(Duration::from_secs(3)).await;
    }
    
    println!("  ❌ Failed to fetch market after 10 attempts");
    None
}