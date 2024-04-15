use std::sync::{Arc, Mutex};

use crate::ledger::block::*;
#[doc(inline)]
use crate::ledger::transaction::*;

use super::super::observer::*;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]


/// Representation of the Blockchain
pub struct Blockchain { 
    pub chain: Vec<Block>, 
    pub difficulty: usize,
    mining_reward: f64, 
    pub is_miner: bool,
    pub temporary_block: Block,
    pub miner_id: String,
    confirmation_pointer:u64,
    event_observer: Arc<Mutex<NetworkEventSystem>>
}

impl Blockchain {
    const INITIAL_DIFFICULTY:usize = 1;
    const NETWORK:&'static str = "network";
    const MAX_TRANSACTIONS:usize = 3;
    const CONFIRMATION_THRESHOLD:usize = 5;

    /// creates a new Blockchain with only the Genesis Block
    pub fn new(is_miner:bool, miner_id:String) -> Blockchain {
        let mut genesis_block = Block::new(0, 
                                       "".to_string(),
                                       Self::INITIAL_DIFFICULTY, 
                                       Self::NETWORK.to_string(),
                                       0.0);

        genesis_block.mine();
        let hash = genesis_block.hash.clone();

        Blockchain {
            chain: vec![genesis_block],
            difficulty: Self::INITIAL_DIFFICULTY,
            mining_reward: 0.01,
            is_miner,
            miner_id: miner_id.clone(),
            confirmation_pointer: 0,
            temporary_block: Block::new(1,
                                        hash.clone(),
                                        Self::INITIAL_DIFFICULTY,
                                        miner_id.clone(),
                                        0.0),
            event_observer: Arc::new(Mutex::new(NetworkEventSystem::new()))
        }
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn BlockchainObserver>>) {
        self.event_observer.lock().unwrap().add_observer(observer);
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

        if !b.check_hash() {
            return false;
        }

        let prev_hash = self.get_head().hash;
        if b.prev_hash != prev_hash{
            return false
        }

        let _ = self.chain.iter_mut()
            .take(Blockchain::CONFIRMATION_THRESHOLD)
            .for_each(|block| {
                block.add_confirmation();
                let c = block.get_confirmations();
                if c <= Self::CONFIRMATION_THRESHOLD {
                    self.confirmation_pointer += 1;
                }
        });

        self.chain.push(b);
        self.adjust_difficulty(); 
        self.adjust_temporary_block(false);

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

    /// returns the current index of the blockchain
    fn get_current_index(&self) -> usize{
        return self.get_head().index;
    }


    /// returns the head Block the blockchain
    pub fn get_head(&self) -> Block {
        return self.chain.last().unwrap().clone();
    }


    /// adds a transaction to a temporary block
    /// when the block is full it will be mined
    /// 
    /// **note** this method is only important to miners,
    pub async fn add_transaction(&mut self, t:Transaction) {
        let index = self.temporary_block.add_transaction(t.clone());
        self.event_observer.lock().unwrap().notify_transaction_created(&t).await;
        if self.is_miner && index >= Self::MAX_TRANSACTIONS {
            self.temporary_block.mine();
            self.event_observer.lock().unwrap().notify_block_mined(&self.temporary_block).await;
            self.add_block(self.temporary_block.clone());
            self.adjust_temporary_block(true);
        }
    }

    ///adjust the temporary block based on the state of the blockchain
    ///if the parameter `create` is true then a new block is created
    ///
    ///**Note:** 
    /// does **not** check for transactions 
    /// that already exist in other blocks
    fn adjust_temporary_block(&mut self, create: bool){
        let head = self.get_head();

        if !create {
            self.temporary_block.index = head.index + 1;
            self.temporary_block.prev_hash = head.hash;
            //self.temporary_block.mining_reward = self.mining_reward;
            self.temporary_block.difficulty = self.difficulty;
            return
        }

        self.temporary_block = Block::new(head.index + 1,
                                          head.hash,
                                          self.difficulty.clone(),
                                          self.miner_id.clone(),
                                          self.mining_reward.clone())
    }

}

// =========================== OBSERVER CODE ==================================== //

impl NetworkObserver for Blockchain {
    fn on_block_received(&mut self, block: &Block) -> bool {
        println!("on_block_received event Triggered on BlockChain: {} => Received Block: {:?}", self.miner_id, block.clone());
        // Check if we already have this block
        // If we do return true and stop here
        // else add the block and return true
        self.add_block(block.clone());

        return false; // It's here while we don't have the "contains_block"
    }

    fn on_transaction_received(&mut self, transaction: &Transaction) -> bool {
        println!("on_transaction_received event Triggered on BlockChain: {} => Received Transaction: {:?}", self.miner_id, transaction.clone());
        // Check if we already have this transaction
        // If we do return true and stop here
        // else add the transaction and return true
        self.chain[0].add_transaction(transaction.clone()); // Just here to check if the transaction is being saved or not (TESTING PURPOSES)
        return false; // It's here while we don't have the "contains_transaction"
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use crate::ledger::blockchain::*;

    fn gen_transaction() -> Transaction {
        let strings = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Carlos".to_string(),
            "Diana".to_string(),
            "Luna".to_string(),
        ];

        let mut rng = rand::thread_rng();
        let from = strings[rng.gen_range(0..strings.len())].clone();
        let to = strings[rng.gen_range(0..strings.len())].clone();
        let out = rng.gen_range(4.0..=10.0);
        let _in = rng.gen_range(1.0..=3.0);

        Transaction::new(out,
            from,
            out-_in,
            to)
    }    

    #[test]
    fn test_adding_blocks() {
        let mut blockchain = Blockchain::new(true,"mario".to_string());

        //block 1
        blockchain.add_transaction( gen_transaction());
        blockchain.add_transaction( gen_transaction());
        //block 2
        blockchain.add_transaction( gen_transaction());
        blockchain.add_transaction( gen_transaction());
        //not added
        blockchain.add_transaction( gen_transaction());

        println!("{:#?}", blockchain);
        assert_eq!(blockchain.get_current_index()+1, 3);
    }

}

