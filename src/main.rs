extern crate core;
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

mod p2p{
    pub mod peer;
    pub mod connect;
}

use crate::kademlia::*;
use crate::ledger::blockchain::Blockchain;

fn main() {
    p2p::connect::connect();
    let mut blockchain = Blockchain::new(false);

    //blockchain.add_block("miner2317".to_string());
    //blockchain.add_block("miner2318".to_string());

    println!("{:#?}", blockchain);

    test_network::test();
}
