mod block; // Import block
mod blockchain;  // Import blockchain

use crate::blockchain::Blockchain;
use crate::block::Block;

fn main() {
    let mut blockchain = Blockchain::new();

    blockchain.add_block("First block data".to_string());
    blockchain.add_block("Second block data".to_string());

    println!("{:#?}", blockchain);

    let mut block_to_mine = Block::new(blockchain.get_current_index()
                                               ,"Block data".to_string()
                                               , "prev_hash_value".to_string()
                                               , 0,blockchain.difficulty);

    let difficulty = blockchain.difficulty;
    block_to_mine.mine(difficulty);
}
