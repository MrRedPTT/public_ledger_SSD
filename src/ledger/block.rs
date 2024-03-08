#[doc(inline)]
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha512, Digest};

use crate::ledger::transaction::*;

#[derive(Debug, Clone)]

/// ## BLock
pub struct Block {
    pub hash: String,

    pub index : usize,
    pub timestamp: u64,
    pub prev_hash: String,
    pub nonce: u64,
    pub difficulty : usize,
    pub miner_id : String,
    pub merkle_tree_root: String,

    pub transactions: Vec<Transaction>,
}

impl Block {

    /// creates a new block with a single transaction (the miner reward)
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

            hash: "".to_string(),
            merkle_tree_root: "".to_string()
        };

        block.add_transaction( Transaction::new(miner_reward,
                                                  "network".to_string(),
                                                  miner_reward,
                                                  miner_id));

        return block
    }


    /// mines the block
    ///
    ///- **outputs:**
    ///    returns true when the block is mined with success
    pub fn mine(&mut self) -> bool {
        loop {
            if self.check_hash() {
                self.hash = self.calculate_hash();
                return true;
            }
            
            self.nonce += 1;
        }
    }

    /// checks if the hash of the block is correct with reference to its dificulty
    pub fn check_hash(&self) -> bool {
        let hash = self.calculate_hash();
        return hash.starts_with(&"0".repeat(self.difficulty)) 
    }

    /// returns the hash(sha512) of the block
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

    /// adds a transaction to the block
    /// if the number of transactions exceeds the max ammount and the client is a miner 
    /// then the block is mined
    ///
    /// **outputs:** id of the transaction
    pub fn add_transaction(&mut self, t: Transaction) -> usize{
        self.transactions.push(t);
        return self.transactions.len()
    }

    /// returns a string of all the transactions inside this block 
    pub fn calculate_merkle_tree(&self) -> String{
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
            println!("Block mined! Nonce: {} Hash: {}", 
                     block.nonce, 
                     block.calculate_hash());
        }
    }

}
