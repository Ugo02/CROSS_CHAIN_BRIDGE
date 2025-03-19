use ethers::types::{Address, U256};

#[derive(Debug)]
pub struct DepositEvent {
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub amount: U256,
    pub nonce: U256,
}

#[derive(Debug)]
pub struct DistributionEvent {
    pub token: Address,
    pub to: Address,
    pub amount: U256,
    pub nonce: U256,
}

#[derive(Debug)]
pub struct DepositDetails {
    pub token: String,
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub nonce: i64,
}