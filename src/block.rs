use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha512, Digest}; // Ensure sha2 crate is in Cargo.toml


#[derive(Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub index : usize,
    pub timestamp: u64,
    pub data: String,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub hash: String,
    pub nonce: u64,
    pub difficulty : usize,
}

impl Block {
    pub fn new(index: usize, data: String, prev_hash: String, nonce : u64, difficulty: usize, transactions: Vec<Transaction>) -> Self {
        let block = Block {
            index,
            timestamp : SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data,
            prev_hash,
            transactions,
            hash: String::new(), // Temporary empty string, will be replaced
            nonce,
            difficulty,
        };

        block
    }

    pub fn mine(&mut self, difficulty: usize) {
        loop {
            let hash = self.calculate_hash();
            if hash.starts_with(&"0".repeat(difficulty)) {
                self.hash = hash;
                println!("Block mined! Nonce: {} Hash: {}", self.nonce, self.hash);
                break;
            } else {
                self.nonce += 1;
            }
        }
    }

    fn calculate_hash(&self) -> String {
        // Use Sha512 to hash the concatenated string of data, timestamp, prev_hash and a nonce
        let mut hasher = Sha512::new();
        hasher.update(format!("{}{}{}{}", &self.data, self.timestamp, &self.prev_hash, self.nonce));
        let hash_result = hasher.finalize();
        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join("");
        hash_hex
    }
}