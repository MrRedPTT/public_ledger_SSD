extern crate core;

mod block; // Import block
mod blockchain;  // Import blockChain
mod kademlia{
    pub mod test_network;
    pub mod kademlia;
    pub mod node;
    pub mod k_buckets;
    pub mod key;

    pub mod bucket;
}

use crate::blockchain::Blockchain;
use crate::kademlia::*;
fn main() {
    let mut blockchain = Blockchain::new();

    blockchain.add_block("First block data".to_string());
    blockchain.add_block("Second block data".to_string());

    println!("{:#?}", blockchain);

    test_network::test();
}
