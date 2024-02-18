#[doc(inline)]
use crate::ledger::transaction::*;
use crate::ledger::block::*;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]


/// Representation of the Blockchain
pub struct Blockchain { 
    pub chain: Vec<Block>, 
    pub difficulty: usize,
    mining_reward: f64, 
    pub is_miner: bool
}

impl Blockchain {
    const initial_difficulty:usize = 1;

    /// creates a new Blockchain with only the Genesis Block
    pub fn new(is_miner:bool) -> Blockchain {
        let genesis_block = Block::new(0, 
                                       "".to_string(),
                                       Self::initial_difficulty, 
                                       "network".to_string(),
                                       0.0);

        Blockchain {
            chain: vec![genesis_block],
            difficulty: Self::initial_difficulty,
            mining_reward: 0.01,
            is_miner
        }
    }

    /// adds a block to the blockchain,
    ///
    /// if the `b.prev_hash != blockchain.head.hash` 
    /// the client will ask the network for missing block(s)
    ///
    /// **outputs:**
    /// returns true if the block is successfully added
    ///
    /// **status:** **not fully implemented**
    /// - missing getting other packages from network
    /// - verification is also not fully done
    pub fn add_block(&mut self, b:Block) -> bool{
        let prev_hash = self.chain.last().unwrap().calculate_hash();

        if !b.check_hash() {
            return false;
        }

        self.chain.push(b);
        self.adjust_difficulty(); 
        return true
    }

    /// adjust the difficulty of the hashes
    fn adjust_difficulty(&mut self) {
        let target_time: u64 = 1 * 60; // Target time to mine a block, e.g., 1 minute
        if self.chain.len() <= 1 {
            return; // Don't adjust if only the genesis block exists
        }

        let last_block = &self.chain[self.chain.len() - 1];
        let prev_block = &self.chain[self.chain.len() - 2];

        let actual_time = last_block.timestamp - prev_block.timestamp;

        // Adjust difficulty based on the actual mining time of the last block
        if actual_time < target_time / 2 {
            self.difficulty += 1; // Increase difficulty if block mined too quickly
        } else if actual_time > target_time * 2 && self.difficulty > 1 {
            self.difficulty -= 1; // Decrease difficulty if block mined too slowly, but never below 1
        }
    }

    fn get_current_index(&mut self) -> usize{
        return self.chain.len();
    }


    /// returns the most recent Block of the blockchain
    pub fn get_head(&self) -> Block {
        return self.chain.last().unwrap().clone();
    }


    /// adds a transaction to a temporary block
    /// when the block is full it will be mined
    /// 
    /// **status:** not impleemnted
    ///
    /// **note** this method is only important to miners,
    pub fn add_transaction(t:Transaction) {}

}
