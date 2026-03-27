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
        
        // Create table with high prices instead of low prices
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_results (
                id SERIAL PRIMARY KEY,
                symbol VARCHAR(10) NOT NULL,
                market_id VARCHAR(100) NOT NULL,
                title VARCHAR(255) NOT NULL,
                start_up_price DOUBLE PRECISION NOT NULL,
                start_down_price DOUBLE PRECISION NOT NULL,
                high_up_price DOUBLE PRECISION NOT NULL,
                high_down_price DOUBLE PRECISION NOT NULL,
                last_up_price DOUBLE PRECISION NOT NULL,
                last_down_price DOUBLE PRECISION NOT NULL,
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
        start_up: f64,
        start_down: f64,
        high_up: f64,
        high_down: f64,
        last_up: f64,
        last_down: f64,
        result: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO market_results (
                symbol, market_id, title,
                start_up_price, start_down_price,
                high_up_price, high_down_price,
                last_up_price, last_down_price,
                result
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(symbol)
        .bind(market_id)
        .bind(title)
        .bind(start_up)
        .bind(start_down)
        .bind(high_up)
        .bind(high_down)
        .bind(last_up)
        .bind(last_down)
        .bind(result)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}