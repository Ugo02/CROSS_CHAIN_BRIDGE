use ethers::{ providers:: Middleware, types::{Address, U64}, signers::Wallet,contract::Contract, middleware::SignerMiddleware};
use eyre::Result;
use sqlx:: PgPool;
use std::{sync::Arc, time::Duration};
use k256::ecdsa::SigningKey;

use crate::{database, utils};

pub async fn process_pending_transactions<M>(
    pool: PgPool,
    sepolia_provider: Arc<M>,
    holesky_provider: Arc<M>,
    sepolia_bridge: Address,
    holesky_bridge: Address,
) -> Result<()> 
where
    M: Middleware + 'static,
{
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    let abi = utils::load_abi();

    dotenv::dotenv().ok();
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in .env");
    let wallet: Wallet<SigningKey> = private_key.parse().expect("Invalid private key");
    
    loop {
        interval.tick().await;
        
        let pending_transactions = database::get_pending_transactions(&pool).await?;
        
        for (nonce, chain) in pending_transactions {
            let deposit = database::get_deposit_details(&pool, nonce).await?;
            
            if let Some(deposit) = deposit {
                println!("Processing pending transaction with nonce: {}", nonce);
                println!("Deposit details: {:?}", deposit);
                
                let (source_provider, dest_provider, dest_bridge) = if chain == "Sepolia" {
                    (sepolia_provider.clone(), holesky_provider.clone(), holesky_bridge)
                } else {
                    (holesky_provider.clone(), sepolia_provider.clone(), sepolia_bridge)
                };
                
                let block_number = source_provider.get_block_number().await?;
                wait_for_finality(source_provider.clone(), block_number, 2).await?;
                
                let dest_contract = Contract::new(dest_bridge, abi.clone(), dest_provider.clone());
                
                let distribute_tx = dest_contract.method::<_, ()>(
                    "distribute",
                    (deposit.token, deposit.to, deposit.amount, deposit.nonce),
                )?;
                
                let client = SignerMiddleware::new(dest_provider.clone(), wallet.clone());
                let pending_tx = distribute_tx.send().await?;
                let receipt = pending_tx.await?;
                
                if let Some(receipt) = receipt {
                    println!("Distribution transaction mined: {:?}", receipt.transaction_hash);
                } else {
                    eprintln!("Distribution transaction failed to be mined");
                }
            }
        }
    }
}

async fn wait_for_finality<M>(provider: Arc<M>, block_number: U64, confirmations: u64) -> Result<()>
where
    M: Middleware + 'static,
{
    let mut current_block = provider.get_block_number().await?;
    while current_block < block_number + confirmations {
        tokio::time::sleep(Duration::from_secs(15)).await;
        current_block = provider.get_block_number().await?;
    }
    Ok(())
}