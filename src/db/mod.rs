// src/db/mod.rs

use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        // New schema - track only one side that drops below 30%
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS crypto (
                id SERIAL PRIMARY KEY,
                symbol VARCHAR(10) NOT NULL,
                market_id VARCHAR(100) NOT NULL,
                title VARCHAR(255) NOT NULL,
                last_up_price DOUBLE PRECISION NOT NULL,
                last_down_price DOUBLE PRECISION NOT NULL,
                side_below_30 VARCHAR(5),  -- "UP" or "DOWN"
                minutes_left_at_below_30 INT,
                high_from_30 DOUBLE PRECISION DEFAULT 0.0,
                result VARCHAR(10) NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        Ok(Database { pool })
    }
    
    pub async fn save_market_result(
    &self,
    symbol: &str,
    market_id: &str,
    title: &str,
    last_up_price: f64,
    last_down_price: f64,
    side_below_30: Option<&String>,  // Accept reference
    minutes_left_at_below_30: Option<i32>,
    high_from_30: f64,
    result: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO crypto (
            symbol, market_id, title,
            last_up_price, last_down_price,
            side_below_30, minutes_left_at_below_30, high_from_30,
            result
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(symbol)
    .bind(market_id)
    .bind(title)
    .bind(last_up_price)
    .bind(last_down_price)
    .bind(side_below_30)  // Option<&String> works with sqlx
    .bind(minutes_left_at_below_30)
    .bind(high_from_30)
    .bind(result)
    .execute(&self.pool)
    .await?;
    
    Ok(())
}
}