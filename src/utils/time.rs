use chrono::Utc;

pub fn now_ts()-> i64{
    Utc::now().timestamp()
}

pub const INTERVAL_15MINS: i64 = 900;

// current window ts
pub fn current_window_ts(now:i64)->i64 {
    now - (now%INTERVAL_15MINS)
}

pub fn window_end_ts(window_start:i64)-> i64 {
    window_start + INTERVAL_15MINS
}

// generate slug
pub fn generate_slug(symbol:&str,window_ts: i64)-> String {
    format!("{}-updown-15m-{}",symbol.to_lowercase(),window_ts)
}