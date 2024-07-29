use warp::Filter;
use std::sync::{Arc, Mutex};
use log::info;
use crate::blockchain::Blockchain;

pub type BlockchainRef = Arc<Mutex<Blockchain>>;

pub async fn serve(blockchain: BlockchainRef) {
    let blockchain_filter = warp::any().map(move || blockchain.clone());

    let add_transaction = warp::post()
        .and(warp::path!("transaction"))
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .map(|transaction: serde_json::Value, blockchain: BlockchainRef| {
            let mut blockchain = blockchain.lock().unwrap();
            let sender = transaction["sender"].as_str().unwrap().to_string();
            let receiver = transaction["receiver"].as_str().unwrap().to_string();
            let amount = transaction["amount"].as_u64().unwrap();
            blockchain.add_transaction(sender, receiver, amount);
            info!("Transaction added successfully");
            warp::reply::json(&*blockchain)
        });

    let mine_block = warp::post()
        .and(warp::path!("mine" / usize))
        .and(blockchain_filter.clone())
        .map(|difficulty: usize, blockchain: BlockchainRef| {
            let mut blockchain = blockchain.lock().unwrap();
            info!("Received mining request. Difficulty: {}", difficulty);
            blockchain.mine_block(difficulty);
            info!("Mining completed. Returning updated blockchain.");
            warp::reply::json(&*blockchain)
        });

    let get_chain = warp::get()
        .and(warp::path("chain"))
        .and(blockchain_filter.clone())
        .map(|blockchain: BlockchainRef| {
            let blockchain = blockchain.lock().unwrap();
            info!("Returning current blockchain state");
            warp::reply::json(&*blockchain)
        });

    let routes = add_transaction.or(mine_block).or(get_chain);
    
    info!("Server starting on http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}