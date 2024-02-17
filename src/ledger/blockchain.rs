use crate::ledger::block::*;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]
pub struct Blockchain { // This is like a class and init
    pub chain: Vec<Block>, // Vector is like a list but more strongly typed
    pub difficulty: usize, //Difficulty
    mining_reward: f64, // Mining reward
    pub is_miner: bool
}

impl Blockchain {
    const initial_difficulty:usize = 1;

    pub fn new(is_miner:bool) -> Self {
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

    // &mut denotes a mutable reference to a value it allows us to borrow a value and modify it
    // without taking ownership

    pub fn add_block(&mut self, b:Block) -> bool{
        let prev_hash = self.chain.last().unwrap().calculate_hash();

        if !b.check_hash() {
            return false;
        }

        self.chain.push(b);
        self.adjust_difficulty(); // Adjust the difficulty after adding the new block
        return true
    }

    // This function is made to adjust the difficulty of the hashes
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

    pub fn get_head(&self) -> Block {
        return self.chain.last().unwrap().clone();
    }
}
