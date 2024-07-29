use std::fs;
use std::path::Path;
use log::{info, error};
use crate::blockchain::Blockchain;
use crate::network::BlockchainRef;

pub fn save_blockchain(blockchain: &Blockchain) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string(blockchain)?;
    fs::write("blockchain.json", serialized)?;
    info!("Blockchain saved to file");
    Ok(())
}

pub fn load_blockchain() -> Result<Blockchain, Box<dyn std::error::Error>> {
    if !Path::new("blockchain.json").exists() {
        return Err("Blockchain file not found".into());
    }
    let contents = fs::read_to_string("blockchain.json")?;
    let blockchain: Blockchain = serde_json::from_str(&contents)?;
    info!("Blockchain loaded from file");
    Ok(blockchain)
}

pub async fn save_blockchain_periodically(blockchain: BlockchainRef) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        let blockchain = blockchain.lock().unwrap();
        if let Err(e) = save_blockchain(&*blockchain) {
            error!("Failed to save blockchain: {}", e);
        }
    }
}