use ethers::{providers::{StreamExt, PubsubClient, Middleware}, types::{Address, Filter, Log}, abi::{EventExt, RawLog}};
use eyre::Result;
use sqlx::PgPool;
use std::sync::Arc;
use futures_util::pin_mut;

use crate::{database, utils, models::{DepositEvent, DistributionEvent}};

pub async fn listen_for_events<M>(
    chain_name: &str,
    provider: M,
    local_bridge: Address,
    remote_bridge: Address,
    pool: PgPool,
    is_source_chain: bool,
) -> Result<()>
where
    M: Middleware + 'static,
    M::Provider: PubsubClient + 'static,
{
    let provider = Arc::new(provider);
    let abi = utils::load_abi();
    
    let deposit_event = abi.event("Deposit").expect("Deposit event not found in ABI");
    let distribution_event = abi.event("Distribution").expect("Distribution event not found in ABI");

    let deposit_filter = Filter::new().address(local_bridge).event(&deposit_event.abi_signature());
    let distribution_filter = Filter::new().address(local_bridge).event(&distribution_event.abi_signature());
    
    let latest_block = provider.get_block_number().await?;
    println!("Starting {} listener from block {}", chain_name, latest_block);
    
    let deposit_logs_stream = provider.subscribe_logs(&deposit_filter).await?;
    let distribution_logs_stream = provider.subscribe_logs(&distribution_filter).await?;

    let deposit_handle = process_deposit_events(
        chain_name, 
        deposit_logs_stream, 
        provider.clone(), 
        pool.clone(),
        is_source_chain,
    );

    let distribution_handle = process_distribution_events(
        chain_name, 
        distribution_logs_stream, 
        provider.clone(), 
        pool.clone(),
    );
    
    tokio::try_join!(deposit_handle, distribution_handle)?;
    
    Ok(())
}

async fn process_deposit_events<M>(
    chain_name: &str,
    mut events: impl StreamExt<Item = Log>,
    provider: Arc<M>,
    pool: PgPool,
    is_source_chain: bool,
) -> Result<()> 
where
    M: Middleware + 'static,
{
    let abi = utils::load_abi();
    let deposit_event = abi.event("Deposit").expect("Deposit event not found in ABI");

    pin_mut!(events);
    while let Some(log) = events.next().await {       
        println!("Received Deposit event on {}: {:?}", chain_name, log);
        
        let raw_log = RawLog {
            topics: log.topics,
            data: log.data.to_vec(),
        };

        let decoded_log = deposit_event.parse_log(raw_log).expect("Failed to decode Deposit event");

        let token = decoded_log.params[0].value.clone().into_address().unwrap();
        let from = decoded_log.params[1].value.clone().into_address().unwrap();
        let to = decoded_log.params[2].value.clone().into_address().unwrap();
        let amount = decoded_log.params[3].value.clone().into_uint().unwrap();
        let nonce = decoded_log.params[4].value.clone().into_uint().unwrap();

        let event = DepositEvent { token, from, to, amount, nonce };
        
        database::store_deposit_event(&pool, &event, chain_name).await?;
        
        if is_source_chain {
            database::insert_pending_transaction(&pool, event.nonce.as_u64()).await?;
        }
    }
    
    Ok(())
}

async fn process_distribution_events<M>(
    chain_name: &str,
    mut events: impl StreamExt<Item = Log>,
    provider: Arc<M>,
    pool: PgPool,
) -> Result<()> 
where
    M: Middleware + 'static,
{
    let abi = utils::load_abi();
    let distribution_event = abi.event("Deposit").expect("Deposit event not found in ABI");

    pin_mut!(events);
    while let Some(log) = events.next().await {
        println!("Received Distribution event on {}: {:?}", chain_name, log);
        
        let raw_log = RawLog {
            topics: log.topics,
            data: log.data.to_vec(),
        };
        
        let decoded_log = distribution_event.parse_log(raw_log).expect("Failed to decode Distribution event");
        
        let token = decoded_log.params[0].value.clone().into_address().unwrap();
        let to = decoded_log.params[1].value.clone().into_address().unwrap();
        let amount = decoded_log.params[2].value.clone().into_uint().unwrap();
        let nonce = decoded_log.params[3].value.clone().into_uint().unwrap();
        
        let event = DistributionEvent { token, to, amount, nonce };
        
        database::store_distribution_event(&pool, &event).await?;
        database::mark_transaction_processed(&pool, event.nonce.as_u64()).await?;
    }
    Ok(())
}