use std::sync::{Arc, Mutex};
use tokio::task;
use log::info;
use crate::blockchain::Blockchain;
use crate::network::{serve, BlockchainRef};
use crate::storage::{load_blockchain, save_blockchain_periodically};

mod blockchain;
mod network;
mod storage;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting blockchain application");

    let blockchain = Arc::new(Mutex::new(load_blockchain().unwrap_or_else(|_| {
        info!("No existing blockchain found. Creating a new one.");
        Blockchain::new()
    })));
    let blockchain_ref: BlockchainRef = Arc::clone(&blockchain);

    task::spawn(async move {
        save_blockchain_periodically(blockchain_ref).await;
    });

    info!("Blockchain initialized. Starting server...");
    serve(blockchain).await;
}