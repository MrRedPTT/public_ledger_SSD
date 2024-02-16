use crate::block::{Block, Transaction};

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]
pub struct Blockchain { // This is like a class and init
    pub chain: Vec<Block>, // Vector is like a list but more strongly typed
    pub difficulty: usize, //Difficulty
    mining_reward: f64, // Mining reward
}

impl Blockchain {
    pub fn new() -> Self {
        let defaut_difficulty = 1;
        let genesis_transactions = Vec::new();
        let genesis_block = Block::new(0,"Genesis block".to_string(), "".to_string(), 0, defaut_difficulty, genesis_transactions);
        Blockchain {
            chain: vec![genesis_block],
            difficulty: defaut_difficulty, //Default difficulty
            mining_reward: 0.01,
        }
    }

    // &mut denotes a mutable reference to a value it allows us to borrow a value and modify it
    // without taking ownership
    pub fn add_block(&mut self, data: String, miner_address: String) {

        let prev_hash = self.chain.last().unwrap().hash.clone();
        let transactions = self.reward_miner(miner_address);

        let mut new_block = Block::new(self.get_current_index(),data, prev_hash, 0, self.difficulty, transactions);

        new_block.mine(self.difficulty); // This needs to be changed when we have p2p connection so that it asks for the block to be mined

        self.chain.push(new_block);

        self.adjust_difficulty(); // Adjust the difficulty after adding the new block
    }

    pub fn reward_miner(&mut self, miner_address: String) -> Vec<Transaction> {
        let reward_transaction = Transaction {
            from: "network".to_string(),
            to: miner_address,
            amount: self.mining_reward, // Define mining_reward as part of Blockchain
        };

        vec![reward_transaction] // This is a return
    }
    /* Something like this
    pub fn async add_block(&mut self, data: String) {
        let prev_hash = self.chain.last().unwrap().hash.clone();


        let mut new_block = Block::new(self.get_current_index(),data, prev_hash, 0, self.difficulty);

        new_block.mine_async(self.difficulty).await;

        self.chain.push(new_block);

        self.adjust_difficulty(); // Adjust the difficulty after adding the new block
    }*/


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

    pub fn get_current_index(&mut self) -> usize{
        return self.chain.len();
    }
}