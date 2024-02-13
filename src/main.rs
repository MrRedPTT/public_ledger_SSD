mod block; // Import block
mod blockchain;  // Import blockChain

use crate::blockchain::Blockchain;
fn main() {
    let mut blockchain = Blockchain::new();

    blockchain.add_block("First block data".to_string());
    blockchain.add_block("Second block data".to_string());

    println!("{:#?}", blockchain);
}
