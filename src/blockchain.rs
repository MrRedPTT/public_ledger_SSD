use crate::block::Block;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]
pub struct Blockchain { // This is like a class and init
    pub chain: Vec<Block>, // Vector is like a list but more strongly typed
    pub difficulty: usize, //Difficulty
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new("Genesis block".to_string(), "".to_string(), 0);
        Blockchain {
            chain: vec![genesis_block],
            difficulty: 4, //Default difficulty
        }
    }

    // &mut denotes a mutable reference to a value it allows us to borrow a value and modify it
    // without taking ownership
    pub fn add_block(&mut self, data: String) {
        let prev_hash = self.chain.last().unwrap().hash.clone();
        let next_nonce = self.chain.last().unwrap().nonce.clone() + 1;
        let new_block = Block::new(data, prev_hash, next_nonce);

        self.chain.push(new_block);
        self.adjust_difficulty(); // Adjust the difficulty after adding the new block
    }

    // This function is made to adjust the difficulty of the hashes
    pub fn adjust_difficulty(&mut self) {
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
}