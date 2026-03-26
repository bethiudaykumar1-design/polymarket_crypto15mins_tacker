
pub struct MarketConfig {
    pub symbol: String,
    pub slug_prefix : String,
    pub interval: i64,
    pub active: bool
}


impl MarketConfig {
    pub fn new(symbol:&str,slug_prefix:&str)->Self {
        Self {
            symbol: symbol.to_string(),
            slug_prefix: slug_prefix.to_string(),
            interval: 900,
            active: true
        }
    }

    pub fn generate_slug(&self,window_ts:i64)->String {
        format!("{}-updown-15m-{}",self.slug_prefix.to_lowercase(),window_ts)
    }

}

pub fn get_all_markets() -> Vec<MarketConfig> {
    vec![
        MarketConfig::new("BTC", "btc"),
        MarketConfig::new("ETH", "eth"),
        MarketConfig::new("SOL", "sol"),
        MarketConfig::new("XRP", "xrp"),
        // MarketConfig::new("DOGE", "doge"),
        // MarketConfig::new("HYPE", "hype"),
        // MarketConfig::new("BNB", "bnb"),
    ]
}

// helper to get market by symbol
pub fn get_market_by_symbol(symbol :&str)->Option<MarketConfig>{
    get_all_markets()
    .into_iter()
    .find(|m| m.symbol==symbol)
}