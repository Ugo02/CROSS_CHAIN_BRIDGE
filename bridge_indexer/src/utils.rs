use ethers::abi::Abi;
use std::fs;

pub fn load_abi() -> Abi {
    let abi_json = fs::read_to_string("abis/Bridge.json").expect("Failed to read ABI file.");
    serde_json::from_str(&abi_json).expect("Failed to parse ABI.")
}