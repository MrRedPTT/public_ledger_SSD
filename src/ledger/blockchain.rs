#[doc(inline)]
use std::sync::{Arc, Mutex};
use crate::ledger::block::*;
use crate::ledger::heads::*;
use crate::ledger::transaction::*;
use super::super::observer::*;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]


/// Representation of the Blockchain
pub struct Blockchain { 
    pub chain: Vec<Block>, 
    pub heads: Heads,
    pub difficulty: usize,
    mining_reward: f64, 
    pub is_miner: bool,
    pub temporary_block: Block,
    pub miner_id: String,
    event_observer: Arc<Mutex<NetworkEventSystem>>
}

// =========================== BLOCKCHAIN CODE ==================================== //
/// Implementation of the basic BlockChain methods
impl Blockchain {
    const INITIAL_DIFFICULTY:usize = 1;
    const NETWORK:&'static str = "network";
    const MAX_TRANSACTIONS:usize = 3;
    const CONFIRMATION_THRESHOLD:usize = 2;

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
            chain: vec![],
            heads: Heads::new(vec![genesis_block], Self::CONFIRMATION_THRESHOLD),
            difficulty: Self::INITIAL_DIFFICULTY,
            mining_reward: 0.01,
            is_miner,
            miner_id: miner_id.clone(),
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
    /// This method assumes that blocks will **not** come out of order
    /// 
    ///
    /// **outputs:**
    /// returns true if the block is successfully added
    ///
    /// ** TODO: **
    /// - verification is also not fully done
    pub fn add_block(&mut self, b:Block) -> bool{
        if !b.check_hash() {
            return false;
        }

        //check if new block fits in heads
        let f = self.heads.add_block(b.clone());
        // if not then is it a new head ?
        if !f {
            if self.chain.last().unwrap().clone().hash == b.clone().prev_hash {
                self.heads.add_head(vec![b]);
            }
            else {
                return false
            }
        }

        match self.heads.get_confirmed() {
            Some(confirmed_block) => {
                self.heads.prune(confirmed_block.prev_hash.clone());
                self.chain.push(confirmed_block);
            }
            None => {
            }
        }
        self.heads.reorder();
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

        let m = self.heads.get_main();
        let last_block = &m[m.len() - 1];
        let prev_block = &m[m.len() - 2];

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
        return self.heads.get_main().len() + self.chain.len();
    }


    /// returns the head Block the blockchain
    pub fn get_head(&self) -> Block {
        return self.heads.get_main().last().unwrap().clone();
    }


    /// adds a transaction to a temporary block
    /// when the block is full it will be mined
    /// 
    /// ** Note ** this method is only important to miners,
    pub async fn add_transaction(&mut self, t:Transaction) {
        let index = self.temporary_block.add_transaction(t.clone());
        self.event_observer.lock().unwrap().notify_transaction_created(&t).await;
        if self.is_miner && index >= Self::MAX_TRANSACTIONS +1 {
            self.temporary_block.mine();
            self.event_observer.lock().unwrap().notify_block_mined(&self.temporary_block).await;
            self.add_block(self.temporary_block.clone());
            self.adjust_temporary_block(true);
        }
    }

    ///adjust the temporary block based on the state of the blockchain
    ///if the parameter `create` is true then a new block is created
    ///
    ///** TODO:** 
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

    //TODO: To remove
    pub fn get_block_by_id(&self, id: usize) -> Option<Block> {
        if id >= self.chain.len() {
                return None;
        }
        return Some(self.chain[id].clone());
    }

    //TODO: Implement
    pub fn get_block_by_hash(&self, hash: String) -> Option<Block> {
        return None;
    }

    /// Check if a block can be added to the block chain
    //TODO: Implement
    pub fn can_add_block(&self, b: Block) -> bool {
        return false;
    }

    //TODO: Implement
    pub fn can_mine(&self) -> bool {
        return false;
    }

    //TODO: Implement
    pub fn mine(&self) -> bool {
        return false;
    }

}

// =========================== OBSERVER CODE ==================================== //
// Implementation of the blockchain events
impl NetworkObserver for Blockchain {
    ///
    fn on_block_received(&mut self, block: &Block) -> bool {
        println!("on_block_received event Triggered on BlockChain: {} => Received Block: {:?}", self.miner_id, block.clone());

        // Check if we already have this block
        // If we do return true and stop here
        // else add the block and return false
        let b = self.get_block_by_hash(block.hash.clone());
        match b {
            Some(x) => {
                if x.hash == block.hash {
                    return true;
                }
                else {
                    // TODO wtf happens when block is not the same as the one we got
                    panic!("oh no!");
                }
            }
            None => {
                    self.add_block(block.clone()); 
                    return false;
            },
        }
    }

    fn on_transaction_received(&mut self, transaction: &Transaction) -> bool {
        println!("on_transaction_received event Triggered on BlockChain: {} => Received Transaction: {:?}", self.miner_id, transaction.clone());
        // TODO check if transaction was already added
        // Check if we already have this transaction
        // If we do return true and stop here
        // else add the transaction and return false
        //self.add_transaction(transaction.clone()); // Just here to check if the transaction is being saved or not (TESTING PURPOSES)
        return false; // It's here while we don't have the "contains_transaction"
    }
}

// =============================== TESTS ======================================== //
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

    #[tokio::test]
    async fn test_adding_blocks() {
        let mut blockchain = Blockchain::new(true,"mario".to_string());

        for i in 0..4 {
            for _ in 1..Blockchain::MAX_TRANSACTIONS {
                blockchain.add_transaction( gen_transaction()).await;
            }
            println!("mined block {:#?}", i);
        }

        println!("{:#?}", blockchain);
        assert_eq!(blockchain.get_current_index()+1, 4);
    }

    #[tokio::test]
    async fn test_branching() {
        let mut bc = Blockchain::new(true,"mario".to_string());

        for _i in 0..2 {
            for _ in 1..Blockchain::MAX_TRANSACTIONS {
                bc.add_transaction( gen_transaction()).await;
            }
        }
        let h = bc.get_head();

        //make a new block
        let mut b = Block::new(h.index+1,
            h.hash, bc.difficulty.clone(),"wario".to_string(),
            100.00);

        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            b.add_transaction( gen_transaction());
        }
        b.mine();

        // new block was mined
        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            bc.add_transaction( gen_transaction()).await;
        }
        
        println!("{:#?}",bc);
        //put other block in the bc
        bc.add_block(b);

        

        println!("{:#?}",bc);
        assert_eq!(bc.heads.num() , 2);
    }

    #[tokio::test]
    async fn test_prunning() {
        let mut bc = Blockchain::new(true,"mario".to_string());

        for _i in 0..2 {
            for _ in 1..Blockchain::MAX_TRANSACTIONS {
                bc.add_transaction( gen_transaction()).await;
            }
        }
        let h = bc.get_head();

        //make a new block
        let mut b = Block::new(h.index+1,
            h.hash, bc.difficulty.clone(),"wario".to_string(),
            100.00);

        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            b.add_transaction( gen_transaction());
        }
        b.mine();

        // new block was mined
        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            bc.add_transaction( gen_transaction()).await;
        }
        
        //put other block in the bc
        bc.add_block(b);

        
        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            bc.add_transaction( gen_transaction()).await;
        }
        for _ in 1..Blockchain::MAX_TRANSACTIONS {
            bc.add_transaction( gen_transaction()).await;
        }

        println!("{:#?}",bc);
        assert_eq!(bc.heads.num() , 1);
    }
}

