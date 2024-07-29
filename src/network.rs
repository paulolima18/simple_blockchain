use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;
use crate::blockchain::Blockchain;
use crate::p2p::P2PNetwork;
use std::net::SocketAddr;

pub type BlockchainRef = Arc<Mutex<Blockchain>>;

pub async fn serve(blockchain: BlockchainRef, p2p_network: Arc<P2PNetwork>, addr: impl Into<SocketAddr>) {
    let addr = addr.into(); // Convert to SocketAddr once
    let blockchain_filter = warp::any().map(move || blockchain.clone());
    let p2p_filter = warp::any().map(move || p2p_network.clone());

    let add_transaction = warp::post()
        .and(warp::path!("transaction"))
        .and(warp::body::json())
        .and(blockchain_filter.clone())
        .and(p2p_filter.clone())
        .and_then(|transaction: serde_json::Value, blockchain: BlockchainRef, p2p: Arc<P2PNetwork>| async move {
            let mut blockchain = blockchain.lock().await;
            let sender = transaction["sender"].as_str().unwrap().to_string();
            let receiver = transaction["receiver"].as_str().unwrap().to_string();
            let amount = transaction["amount"].as_u64().unwrap();
            blockchain.add_transaction(sender.clone(), receiver.clone(), amount);
            
            // Broadcast the new transaction to peers
            let transaction = crate::blockchain::Transaction { sender, receiver, amount };
            p2p.broadcast_transaction(transaction).await;
            
            info!("Transaction added and broadcasted successfully");
            Ok::<_, warp::Rejection>(warp::reply::json(&*blockchain))
        });

    let mine_block = warp::post()
        .and(warp::path!("mine" / usize))
        .and(blockchain_filter.clone())
        .and(p2p_filter.clone())
        .and_then(|difficulty: usize, blockchain: BlockchainRef, p2p: Arc<P2PNetwork>| async move {
            let mut blockchain = blockchain.lock().await;
            info!("Received mining request. Difficulty: {}", difficulty);
            blockchain.mine_block(difficulty);
            
            // Broadcast the new block to peers
            let new_block = blockchain.chain.last().unwrap().clone();
            p2p.broadcast_block(new_block).await;
            
            info!("Mining completed and new block broadcasted. Returning updated blockchain.");
            Ok::<_, warp::Rejection>(warp::reply::json(&*blockchain))
        });

    let get_chain = warp::get()
        .and(warp::path("chain"))
        .and(blockchain_filter.clone())
        .and_then(|blockchain: BlockchainRef| async move {
            let blockchain = blockchain.lock().await;
            info!("Returning current blockchain state");
            Ok::<_, warp::Rejection>(warp::reply::json(&*blockchain))
        });

    let routes = add_transaction.or(mine_block).or(get_chain);
    
    info!("Server starting on http://{}", addr);
    warp::serve(routes).run(addr).await;
}