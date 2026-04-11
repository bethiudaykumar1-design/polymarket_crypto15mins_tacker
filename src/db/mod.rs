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
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS test (
                id SERIAL PRIMARY KEY,
                symbol VARCHAR(10) NOT NULL,
                market_id VARCHAR(100) NOT NULL,
                title VARCHAR(255) NOT NULL,
                last_up_price DOUBLE PRECISION NOT NULL,
                last_down_price DOUBLE PRECISION NOT NULL,
                result VARCHAR(10) NOT NULL,
                
                -- UP thresholds
                up_035 BOOLEAN DEFAULT FALSE,
                up_030 BOOLEAN DEFAULT FALSE,
                up_025 BOOLEAN DEFAULT FALSE,
                up_020 BOOLEAN DEFAULT FALSE,
                up_015 BOOLEAN DEFAULT FALSE,
                up_010 BOOLEAN DEFAULT FALSE,
                
                up_from_035 DOUBLE PRECISION DEFAULT 0.0,
                up_from_030 DOUBLE PRECISION DEFAULT 0.0,
                up_from_025 DOUBLE PRECISION DEFAULT 0.0,
                up_from_020 DOUBLE PRECISION DEFAULT 0.0,
                up_from_015 DOUBLE PRECISION DEFAULT 0.0,
                up_from_010 DOUBLE PRECISION DEFAULT 0.0,
                
                up_minutes_035 INT,
                up_minutes_030 INT,
                up_minutes_025 INT,
                up_minutes_020 INT,
                up_minutes_015 INT,
                up_minutes_010 INT,
                
                -- DOWN thresholds
                down_035 BOOLEAN DEFAULT FALSE,
                down_030 BOOLEAN DEFAULT FALSE,
                down_025 BOOLEAN DEFAULT FALSE,
                down_020 BOOLEAN DEFAULT FALSE,
                down_015 BOOLEAN DEFAULT FALSE,
                down_010 BOOLEAN DEFAULT FALSE,
                
                down_from_035 DOUBLE PRECISION DEFAULT 0.0,
                down_from_030 DOUBLE PRECISION DEFAULT 0.0,
                down_from_025 DOUBLE PRECISION DEFAULT 0.0,
                down_from_020 DOUBLE PRECISION DEFAULT 0.0,
                down_from_015 DOUBLE PRECISION DEFAULT 0.0,
                down_from_010 DOUBLE PRECISION DEFAULT 0.0,
                
                down_minutes_035 INT,
                down_minutes_030 INT,
                down_minutes_025 INT,
                down_minutes_020 INT,
                down_minutes_015 INT,
                down_minutes_010 INT
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
        result: &str,
        // UP data
        up_035: bool, up_030: bool, up_025: bool, up_020: bool, up_015: bool, up_010: bool,
        up_from_035: f64, up_from_030: f64, up_from_025: f64, up_from_020: f64, up_from_015: f64, up_from_010: f64,
        up_minutes_035: Option<i32>, up_minutes_030: Option<i32>, up_minutes_025: Option<i32>,
        up_minutes_020: Option<i32>, up_minutes_015: Option<i32>, up_minutes_010: Option<i32>,
        // DOWN data
        down_035: bool, down_030: bool, down_025: bool, down_020: bool, down_015: bool, down_010: bool,
        down_from_035: f64, down_from_030: f64, down_from_025: f64, down_from_020: f64, down_from_015: f64, down_from_010: f64,
        down_minutes_035: Option<i32>, down_minutes_030: Option<i32>, down_minutes_025: Option<i32>,
        down_minutes_020: Option<i32>, down_minutes_015: Option<i32>, down_minutes_010: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO test (
                symbol, market_id, title, last_up_price, last_down_price, result,
                up_035, up_030, up_025, up_020, up_015, up_010,
                up_from_035, up_from_030, up_from_025, up_from_020, up_from_015, up_from_010,
                up_minutes_035, up_minutes_030, up_minutes_025, up_minutes_020, up_minutes_015, up_minutes_010,
                down_035, down_030, down_025, down_020, down_015, down_010,
                down_from_035, down_from_030, down_from_025, down_from_020, down_from_015, down_from_010,
                down_minutes_035, down_minutes_030, down_minutes_025, down_minutes_020, down_minutes_015, down_minutes_010
            ) VALUES ($1, $2, $3, $4, $5, $6,
                      $7, $8, $9, $10, $11, $12,
                      $13, $14, $15, $16, $17, $18,
                      $19, $20, $21, $22, $23, $24,
                      $25, $26, $27, $28, $29, $30,
                      $31, $32, $33, $34, $35, $36,
                      $37, $38, $39, $40, $41, $42,
                      $43, $44, $45, $46, $47, $48)
            "#,
        )
        .bind(symbol)
        .bind(market_id)
        .bind(title)
        .bind(last_up_price)
        .bind(last_down_price)
        .bind(result)
        // UP booleans
        .bind(up_035).bind(up_030).bind(up_025).bind(up_020).bind(up_015).bind(up_010)
        // UP from prices
        .bind(up_from_035).bind(up_from_030).bind(up_from_025).bind(up_from_020).bind(up_from_015).bind(up_from_010)
        // UP minutes
        .bind(up_minutes_035).bind(up_minutes_030).bind(up_minutes_025)
        .bind(up_minutes_020).bind(up_minutes_015).bind(up_minutes_010)
        // DOWN booleans
        .bind(down_035).bind(down_030).bind(down_025).bind(down_020).bind(down_015).bind(down_010)
        // DOWN from prices
        .bind(down_from_035).bind(down_from_030).bind(down_from_025).bind(down_from_020).bind(down_from_015).bind(down_from_010)
        // DOWN minutes
        .bind(down_minutes_035).bind(down_minutes_030).bind(down_minutes_025)
        .bind(down_minutes_020).bind(down_minutes_015).bind(down_minutes_010)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}