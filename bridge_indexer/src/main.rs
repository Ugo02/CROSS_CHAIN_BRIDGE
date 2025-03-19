use ethers::{ providers::{Ws, Provider}, types::Address};
use eyre::Result;
use std::{env, str::FromStr, sync::Arc};

mod models;
mod events;
mod database;
mod utils;
mod processor;

use events::listen_for_events;
use processor::process_pending_transactions;
use database::setup_database;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    // Database connection
    let pool = setup_database().await?;
    
    // Chain configurations
    let sepolia_rpc = env::var("SEPOLIA_RPC_URL").expect("SEPOLIA_RPC_URL must be set");
    let holesky_rpc = env::var("HOLESKY_RPC_URL").expect("HOLESKY_RPC_URL must be set");
    let sepolia_bridge_addr = env::var("SEPOLIA_BRIDGE_ADDRESS").expect("SEPOLIA_BRIDGE_ADDRESS must be set");
    let holesky_bridge_addr = env::var("HOLESKY_BRIDGE_ADDRESS").expect("HOLESKY_BRIDGE_ADDRESS must be set");
    
    let sepolia_bridge = Address::from_str(&sepolia_bridge_addr)?;
    let holesky_bridge = Address::from_str(&holesky_bridge_addr)?;
    
    let sepolia_provider = Provider::<Ws>::connect(sepolia_rpc).await?;
    let holesky_provider = Provider::<Ws>::connect(holesky_rpc).await?;

    let sepolia_provider_arc = Arc::new(sepolia_provider.clone());
    let holesky_provider_arc = Arc::new(holesky_provider.clone());
    
    let pool_sepolia = pool.clone();
    let pool_holesky = pool.clone();
    let pool_processor = pool.clone();
    
    let sepolia_handle = tokio::spawn(async move {
        listen_for_events(
            "Sepolia", 
            sepolia_provider,
            sepolia_bridge,
            holesky_bridge,
            pool_sepolia,
            true,
        ).await
    });
    
    let holesky_handle = tokio::spawn(async move {
        listen_for_events(
            "Holesky", 
            holesky_provider,
            holesky_bridge,
            sepolia_bridge,
            pool_holesky,
            true,
        ).await
    });
    
    let processor_handle = tokio::spawn(async move {
        process_pending_transactions(
            pool_processor,
            sepolia_provider_arc,
            holesky_provider_arc,
            sepolia_bridge,
            holesky_bridge,
        ).await
    });
    
    let _ = tokio::try_join!(sepolia_handle, holesky_handle, processor_handle)?;
    
    Ok(())
}