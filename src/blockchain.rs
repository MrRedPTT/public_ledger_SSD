use crate::block::Block;

// Used to apply Debug and Clone traits to the struct, debug allows printing with the use of {:?} or {:#?}
// and Clone allows for structure and its data to duplicated
#[derive(Debug, Clone)]
pub struct Blockchain { // This is like a class and init
    pub chain: Vec<Block>, // Vector is like a list but more strongly typed
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new("Genesis block".to_string(), "".to_string());
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    // &mut denotes a mutable reference to a value it allows us to borrow a value and modify it
    // without taking ownership
    pub fn add_block(&mut self, data: String) {
        let prev_hash = self.chain.last().unwrap().hash.clone();
        let new_block = Block::new(data, prev_hash);
        self.chain.push(new_block);
    }
}