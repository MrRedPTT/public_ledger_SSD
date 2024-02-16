mod block; // Import block
mod blockchain;  // Import blockchain

use crate::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    blockchain.add_block("First block data".to_string(), "miner2317".to_string());
    blockchain.add_block("Second block data".to_string(),"miner2318".to_string());

    println!("{:#?}", blockchain);
}
