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
    pub mod auxi;
}
mod ledger;

use crate::blockchain::Blockchain;
use crate::kademlia::*;
use crate::ledger::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    //blockchain.add_block("miner2317".to_string());
    //blockchain.add_block("miner2318".to_string());

    println!("{:#?}", blockchain);

    test_network::test();
}
