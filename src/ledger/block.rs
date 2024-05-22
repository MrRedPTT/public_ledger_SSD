#[doc(inline)]
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha512};

use crate::{auxi, proto};
use crate::marco::marco::Marco;
use crate::marco::transaction::Transaction;

#[derive(Debug, Clone)]

/// ## Block
pub struct Block {
    pub hash: String,

    pub index : usize,
    pub timestamp: u64,
    pub prev_hash: String,
    pub nonce: u64,
    pub difficulty : usize,
    pub miner_id : String,
    pub merkle_tree_root: String,
    pub(crate) confirmations: usize,

    pub transactions: Vec<Marco>,
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
            confirmations: 0,

            hash: "".to_string(),
            merkle_tree_root: "".to_string()
        };

        block.add_marco(Marco::from_transaction( 
            Transaction::new(miner_reward,
                "network".to_string(),
                miner_reward,
                miner_id)
        ));

        return block
    }

    pub fn proto_to_block(proto_block: proto::Block) -> Self {
        let mut trans: Vec<Marco> = Vec::new();

        for i in &proto_block.transactions {
            trans.push(auxi::transform_proto_to_marco(i));
        }

        Block {
            hash: proto_block.hash.clone(),
            index: proto_block.index as usize,
            timestamp: proto_block.timestamp,
            prev_hash: proto_block.prev_hash.clone(),
            nonce: proto_block.nonce,
            difficulty: proto_block.difficulty as usize,
            miner_id: proto_block.miner_id.clone(),
            merkle_tree_root: proto_block.merkle_tree_root.clone(),
            confirmations: 0,
            transactions: trans,
        }
    }


    /// mines the block
    ///
    ///- **outputs:**
    ///    returns true when the block is mined with success
    pub fn mine(&mut self) -> bool {
        self.calculate_merkle_tree();
        loop {
            self.hash = self.calculate_hash();
            if self.check_hash() {
                return true;
            }
            
            self.nonce += 1;
        }
    }

    /// checks if the hash of the block is correct with reference to its dificulty
    pub fn check_hash(&self) -> bool {
        return self.hash.starts_with(&"0".repeat(self.difficulty))
    }

    /// returns the hash(sha512) of the block
    pub fn calculate_hash(&self) -> String {
        // Use Sha512 to hash the concatenated string of data, timestamp, prev_hash and a nonce
        let mut hasher = Sha512::new();
        hasher.update(format!("{}{}{}{}",
                              self.merkle_tree_root,
                              self.timestamp,
                              &self.prev_hash, 
                              self.nonce));

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
    pub fn add_marco(&mut self, t: Marco) -> usize{
        self.transactions.push(t);
        return self.transactions.len()
    }

    /// returns the root of the merkle tree 
    /// Sets the object property to the result of the computation
    /// returns true if successful 
    pub fn calculate_merkle_tree(&mut self) -> bool {
        fn hash2(s1:String, s2:String) -> String {
            let mut hasher = Sha512::new();
            hasher.update(format!("{}{}",s1,s2));
            let hash_result = hasher.finalize();

            let hash_hex = hash_result.iter()
                .map(|byte| format!("{:02x}",byte))
                .collect::<Vec<String>>()
                .join("");
            hash_hex
        }

        let n_transac = self.transactions.len();

        if n_transac  == 0 {
            return false;
        }
        else if n_transac == 1 {
            self.merkle_tree_root = self.transactions[0].hash();
            return true;
        }

        let mut fin: Vec<String>;


        if n_transac % 2 == 1 && n_transac >= 3 {
            let (first, rest) = self.transactions.split_at(2);
            let a = hash2(first[0].hash(),first[1].hash());
            fin = vec![a];
            fin.extend(rest.iter()
                    .map(|trans| trans.hash() )
                    .collect::<Vec<String>>());
        }else {
            fin = self.transactions.iter()
                    .map(|trans| trans.hash() )
                    .collect::<Vec<String>>();
        }
        
        
        loop {
            if fin.len() == 1 {
                break;
            }
            
            let mut  temp = Vec::new();
            
            for pair in fin.chunks(2) {
                let first = pair[0].clone();
                if pair.len() == 1 {
                    temp.push(first);
                    continue;
                }
                
                let second = pair[1].clone();
                temp.push(hash2(first, second));
            }
           // println!("temp: {:?}", temp);
            fin = temp;
        }
        //println!("temp: {:?}", fin);
        self.merkle_tree_root = fin[0].clone();
        return true;
    }

    pub fn add_confirmation(&mut self) {
        self.confirmations +=1;
    }
    pub fn get_confirmations(&self) -> usize {
        return self.confirmations;
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

        block.add_marco(Marco::from_transaction( Transaction::new(5.0,
                                          "alice".to_string(),
                                          4.5,
                                          "bob".to_string())));
        block.add_marco(Marco::from_transaction(Transaction::new(4.0,
                                          "Carlos".to_string(),
                                          2.0,
                                          "bob".to_string())));

        if block.mine() {
            println!("Block mined! Nonce: {} Hash: {}", 
                     block.nonce, 
                     block.calculate_hash());
        }
    }
}
