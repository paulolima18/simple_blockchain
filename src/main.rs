use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;
use std::net::SocketAddr;
use structopt::StructOpt;
use crate::blockchain::Blockchain;
use crate::network::serve;
use crate::storage::{load_blockchain, save_blockchain_periodically};
use crate::p2p::P2PNetwork;

mod blockchain;
mod network;
mod storage;
mod p2p;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "P", long, default_value = "3030")]
    port: u16,
    #[structopt(short, long)]
    peers: Vec<SocketAddr>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    info!("Starting blockchain application");

    let blockchain = Arc::new(Mutex::new(load_blockchain().unwrap_or_else(|_| {
        info!("No existing blockchain found. Creating a new one.");
        Blockchain::new()
    })));

    let p2p_network = Arc::new(P2PNetwork::new(Arc::clone(&blockchain)));

    let p2p_addr = SocketAddr::from(([127, 0, 0, 1], opt.port + 1000));
    let p2p_network_clone = Arc::clone(&p2p_network);
    tokio::spawn(async move {
        p2p_network_clone.start(p2p_addr).await;
    });

    for peer in opt.peers {
        let p2p_network = Arc::clone(&p2p_network);
        tokio::spawn(async move {
            p2p_network.connect_to_peer(peer).await;
        });
    }

    let blockchain_clone = Arc::clone(&blockchain);
    tokio::spawn(async move {
        save_blockchain_periodically(blockchain_clone).await;
    });

    info!("Blockchain initialized. Starting server...");
    serve(blockchain, p2p_network, ([127, 0, 0, 1], opt.port)).await;
}