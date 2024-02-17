use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha512, Digest}; // Ensure sha2 crate is in Cargo.toml


use crate::ledger::transaction::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub index : usize,
    pub timestamp: u64,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub difficulty : usize,
    pub miner_id : String,
}

impl Block {
    pub fn new(index: usize, 
               prev_hash: String, 
               difficulty: usize, 
               miner_id: String,
               miner_reward:f64) -> Self {
        let mut block = Block {
            index,
            timestamp : SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            prev_hash,
            transactions: Vec::new(),
            nonce : 0,
            difficulty,
            miner_id: miner_id.clone(),
        };
        block.add_transaction( Transaction::new(miner_reward,
                                                  "network".to_string(),
                                                  miner_reward,
                                                  miner_id));

        return block
    }

    pub fn mine(&mut self) -> bool {
        loop {
            if self.check_hash() {
                return true;
            }
            
            self.nonce += 1;
        }
    }

    pub fn check_hash(&self) -> bool {
        let hash = self.calculate_hash();
        return hash.starts_with(&"0".repeat(self.difficulty)) 
    }

    pub fn calculate_hash(&self) -> String {
        // Use Sha512 to hash the concatenated string of data, timestamp, prev_hash and a nonce
        let mut hasher = Sha512::new();
        let trans = self.transactions_to_string();
        hasher.update(format!("{}{}{}{}",trans, self.timestamp, &self.prev_hash, self.nonce));
        let hash_result = hasher.finalize();

        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
        hash_hex
    }

    pub fn add_transaction(&mut self, t: Transaction) -> usize{
        self.transactions.push(t);
        return self.transactions.len()
    }

    pub fn transactions_to_string(&self) -> String{
        return  self.transactions.iter()
            .map(|t| t.to_string())
            .collect::<String>();
    }
}


#[cfg(test)]
mod test {
    use crate::ledger::block::*;
    
    #[test]
    fn test_mining() {
        let mut block = Block::new(1,
                               "".to_string(),
                               3,
                               "test".to_string(),
                               3.5);

        block.add_transaction( Transaction::new(5.0,
                                          "alice".to_string(),
                                          4.5,
                                          "bob".to_string()));
        block.add_transaction( Transaction::new(4.0,
                                          "Carlos".to_string(),
                                          2.0,
                                          "bob".to_string()));

        if block.mine() {
            println!("Block mined! Nonce: {} Hash: {}", block.nonce, block.calculate_hash());
        }
    }

}
