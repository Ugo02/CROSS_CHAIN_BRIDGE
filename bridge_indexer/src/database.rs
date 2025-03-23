use eyre::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::models::{DepositEvent, DistributionEvent, DepositDetails};

pub async fn setup_database() -> Result<PgPool> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;
    Ok(pool)
}

pub async fn store_deposit_event(pool: &PgPool, event: &DepositEvent, chain: &str) -> Result<()> {
    sqlx::query!(
        "INSERT INTO deposits (token, sender, recipient, amount, nonce, chain) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         ON CONFLICT (nonce, chain) DO NOTHING",
        event.token.to_string(),
        event.from.to_string(),
        event.to.to_string(),
        event.amount.as_u128() as i64,
        event.nonce.as_u64() as i64,
        chain,
    )
    .execute(pool)
    .await?;
    
    println!("Stored deposit event with nonce: {} on chain: {}", event.nonce, chain);
    Ok(())
}

pub async fn store_distribution_event(pool: &PgPool, event: &DistributionEvent) -> Result<()> {
    sqlx::query!(
        "INSERT INTO distributions (token, recipient, amount, nonce) 
         VALUES ($1, $2, $3, $4) 
         ON CONFLICT (nonce) DO NOTHING",
        event.token.to_string(),
        event.to.to_string(),
        event.amount.as_u128() as i64,
        event.nonce.as_u64() as i64,
    )
    .execute(pool)
    .await?;
    
    println!("Stored distribution event with nonce: {}", event.nonce);
    Ok(())
}

pub async fn insert_pending_transaction(pool: &PgPool, nonce: u64) -> Result<()> {
    let exists = sqlx::query!(
        "SELECT nonce FROM processed_transactions WHERE nonce = $1",
        nonce as i64
    ).fetch_optional(pool).await?;
    
    if exists.is_none() {
        println!("Added pending transaction with nonce: {}", nonce);
    }    
    Ok(())
}

pub async fn mark_transaction_processed(pool: &PgPool, nonce: u64) -> Result<()> {
    sqlx::query!(
        "INSERT INTO processed_transactions (nonce) VALUES ($1) ON CONFLICT (nonce) DO NOTHING",
        nonce as i64
    ).execute(pool).await?;
    
    println!("Marked transaction with nonce {} as processed", nonce);
    Ok(())
}

pub async fn get_pending_transactions(pool: &PgPool) -> Result<Vec<(u64, String)>> {
    let pending = sqlx::query!(
        "SELECT d.nonce, d.chain FROM deposits d 
         LEFT JOIN processed_transactions pt ON d.nonce = pt.nonce 
         WHERE pt.nonce IS NULL"
    )
    .fetch_all(pool).await?;

    Ok(pending.iter().map(|r| (r.nonce as u64, r.chain.clone().unwrap())).collect())
}

pub async fn get_deposit_details(pool: &PgPool, nonce: u64) -> Result<Option<DepositDetails>> {
    let deposit = sqlx::query_as!(
        DepositDetails,
        "SELECT token, sender as from, recipient as to, amount, nonce FROM deposits WHERE nonce = $1",
        nonce as i64
    )
    .fetch_optional(pool).await?;
    Ok(deposit)
}