use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use log::{info, error};
use crate::blockchain::{Blockchain, Block, Transaction};

type Peers = Arc<Mutex<HashSet<SocketAddr>>>;

#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone derive
enum Message {
    NewBlock(Block),
    NewTransaction(Transaction),
    RequestChain,
    SendChain(Blockchain),
}

#[derive(Debug, Clone)]
pub struct P2PNetwork {
    peers: Peers,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl P2PNetwork {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        P2PNetwork {
            peers: Arc::new(Mutex::new(HashSet::new())),
            blockchain,
        }
    }

    pub async fn start(&self, addr: SocketAddr) {
        let listener = TcpListener::bind(addr).await.unwrap();
        info!("P2P network listening on: {}", addr);

        loop {
            let (socket, peer_addr) = listener.accept().await.unwrap();
            info!("New peer connected: {}", peer_addr);

            let peers = self.peers.clone();
            let blockchain = self.blockchain.clone();

            tokio::spawn(async move {
                Self::handle_connection(socket, peer_addr, peers, blockchain).await;
            });
        }
    }

    pub async fn connect_to_peer(&self, addr: SocketAddr) {
        if let Ok(stream) = TcpStream::connect(addr).await {
            info!("Connected to peer: {}", addr);
            self.peers.lock().await.insert(addr);
            let blockchain = self.blockchain.clone();
            let peers = self.peers.clone();

            tokio::spawn(async move {
                Self::handle_connection(stream, addr, peers, blockchain).await;
            });
        } else {
            error!("Failed to connect to peer: {}", addr);
        }
    }

    async fn handle_connection(mut socket: TcpStream, addr: SocketAddr, peers: Peers, blockchain: Arc<Mutex<Blockchain>>) {
        peers.lock().await.insert(addr);

        loop {
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    info!("Peer disconnected: {}", addr);
                    peers.lock().await.remove(&addr);
                    break;
                }
                Ok(n) => {
                    if let Ok(message) = serde_json::from_slice::<Message>(&buffer[..n]) {
                        Self::handle_message(message, &blockchain, &peers).await;
                    }
                }
                Err(e) => {
                    error!("Error reading from socket: {}", e);
                    peers.lock().await.remove(&addr);
                    break;
                }
            }
        }
    }

    async fn handle_message(message: Message, blockchain: &Arc<Mutex<Blockchain>>, peers: &Peers) {
        match message {
            Message::NewBlock(block) => {
                let mut chain = blockchain.lock().await;
                if block.index == chain.chain.len() && block.previous_hash == chain.chain.last().unwrap().hash() {
                    chain.chain.push(block.clone()); // Clone the block before pushing
                    info!("New block added to the chain");
                    drop(chain); // Release the lock before broadcasting
                    Self::broadcast_message(&Message::NewBlock(block), peers).await;
                }
            }
            Message::NewTransaction(transaction) => {
                let mut chain = blockchain.lock().await;
                chain.add_transaction(transaction.sender.clone(), transaction.receiver.clone(), transaction.amount);
                info!("New transaction added");
                drop(chain); // Release the lock before broadcasting
                Self::broadcast_message(&Message::NewTransaction(transaction), peers).await;
            }
            Message::RequestChain => {
                let chain = blockchain.lock().await;
                Self::broadcast_message(&Message::SendChain((*chain).clone()), peers).await;
            }
            Message::SendChain(new_chain) => {
                let mut chain = blockchain.lock().await;
                if new_chain.chain.len() > chain.chain.len() {
                    *chain = new_chain;
                    info!("Chain updated from peer");
                }
            }
        }
    }

    async fn broadcast_message(message: &Message, peers: &Peers) {
        let serialized = serde_json::to_string(message).unwrap();
        let peers_lock = peers.lock().await;
        for peer in peers_lock.iter() {
            if let Ok(mut stream) = TcpStream::connect(peer).await {
                let _ = stream.write_all(serialized.as_bytes()).await;
            }
        }
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) {
        Self::broadcast_message(&Message::NewTransaction(transaction), &self.peers).await;
    }

    pub async fn broadcast_block(&self, block: Block) {
        Self::broadcast_message(&Message::NewBlock(block), &self.peers).await;
    }
}