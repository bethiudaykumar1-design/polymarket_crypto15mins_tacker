// src/api/resolve.rs

/// Determine the result of a market
/// 
/// # Arguments
/// * `market_id` - Polymarket's internal market ID
/// * `up_price` - Final price of UP token
/// * `down_price` - Final price of DOWN token
/// 
/// # Returns
/// * `"UP"` - If UP token price is higher
/// * `"DOWN"` - If DOWN token price is higher
/// * `"UNKNOWN"` - If we can't determine
pub async fn resolve_result(
    market_id: &str,
    up_price: f64,
    down_price: f64,
) -> String {
    // Try to get official result from Gamma API first
    let url = format!(
        "https://gamma-api.polymarket.com/markets/{}",
        market_id
    );
    
    // println!("🔍 Checking official result for market: {}", market_id);
    
    match reqwest::get(&url).await {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // Check if market is closed
                        if json["closed"].as_bool() == Some(true) {
                            // Get official outcome prices
                            if let Some(prices) = json["outcomePrices"].as_array() {
                                if prices.len() >= 2 {
                                    let p0 = prices[0].as_str()
                                        .unwrap_or("0")
                                        .parse::<f64>()
                                        .unwrap_or(0.0);
                                    let p1 = prices[1].as_str()
                                        .unwrap_or("0")
                                        .parse::<f64>()
                                        .unwrap_or(0.0);
                                    
                                    // println!("📊 Official prices: {:.4} vs {:.4}", p0, p1);
                                    
                                    // Official result based on which price is higher
                                    if p0 > p1 {
                                        // println!("✅ Official result: UP");
                                        return "UP".to_string();
                                    } else if p1 > p0 {
                                        // println!("✅ Official result: DOWN");
                                        return "DOWN".to_string();
                                    } else {
                                        // println!("⚠️ Official prices are equal, checking our data");
                                    }
                                }
                            }
                        } else {
                            // println!("⚠️ Market not closed yet, using our price data");
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Failed to parse Gamma response: {}", e);
                    }
                }
            } else {
                println!("⚠️ Failed to fetch market data, status: {}", res.status());
            }
        }
        Err(e) => {
            println!("⚠️ Failed to reach Gamma API: {}", e);
        }
    }
    
    // Fallback: Use our tracked prices
    // println!("📊 Using tracked prices: UP={:.4}, DOWN={:.4}", up_price, down_price);
    
    let result = if up_price > down_price {
        "UP"
    } else if down_price > up_price {
        "DOWN"
    } else {
        // If equal, check if we have any historical data?
        // println!("⚠️ Prices are equal, defaulting to UP");
        "UP"  // Default if somehow equal
    };
    
    // println!("✅ Resolved result: {}", result);
    result.to_string()
}