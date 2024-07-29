use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use log::{info, warn};
use std::time::{Instant, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: usize,
    pub timestamp: String,
    pub proof: u64,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn hash(&self) -> String {
        let block_data = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(block_data.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
        };
        blockchain.add_block(0, "0".to_string());
        blockchain
    }

    pub fn add_block(&mut self, proof: u64, previous_hash: String) {
        let block = Block {
            index: self.chain.len(),
            timestamp: Utc::now().to_rfc3339(),
            proof,
            previous_hash,
            transactions: std::mem::take(&mut self.pending_transactions),
        };

        self.chain.push(block);
        info!("New block added to the chain. Chain length: {}", self.chain.len());
    }

    pub fn add_transaction(&mut self, sender: String, receiver: String, amount: u64) {
        self.pending_transactions.push(Transaction { sender, receiver, amount });
        info!("New transaction added. Pending transactions: {}", self.pending_transactions.len());
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let last_block = self.last_block().unwrap();
        let proof = self.proof_of_work(last_block, difficulty);
        let previous_hash = last_block.hash();

        info!("Mining successful. Proof: {}, Previous hash: {}", proof, previous_hash);

        self.add_block(proof, previous_hash);
    }

    fn proof_of_work(&self, last_block: &Block, difficulty: usize) -> u64 {
        let start_time = Instant::now();
        let max_attempts = 1_000_000; // Limit the number of attempts
        let mut proof = 0;
        
        info!("Starting proof-of-work. Difficulty: {}", difficulty);

        for attempt in 0..max_attempts {
            if self.valid_proof(last_block, proof, difficulty) {
                let duration = start_time.elapsed();
                info!("Proof found after {} attempts. Time taken: {:?}", attempt, duration);
                return proof;
            }
            proof += 1;

            if attempt % 100_000 == 0 {
                info!("Mining in progress. Attempts: {}", attempt);
            }

            if start_time.elapsed() > Duration::from_secs(30) {
                warn!("Mining timed out after 30 seconds");
                return proof; // Return the best proof found so far
            }
        }

        warn!("Failed to find proof within {} attempts", max_attempts);
        proof // Return the best proof found so far
    }

    fn valid_proof(&self, last_block: &Block, proof: u64, difficulty: usize) -> bool {
        let guess = format!("{}{}{}", last_block.proof, last_block.hash(), proof);
        let guess_hash = Sha256::digest(guess.as_bytes());
        &guess_hash[0..difficulty] == &vec![0u8; difficulty]
    }

    fn last_block(&self) -> Option<&Block> {
        self.chain.last()
    }
}