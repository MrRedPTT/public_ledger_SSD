extern crate core;

use std::env;
use crate::kademlia::node::Node;
use crate::p2p::peer::Peer;

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

    pub mod protocol;


}


// HOW TO TEST
// Open 3 terminals
// The first one (server) run => cargo run -- 1
// In the second (client) run => cargo run -- 2
// And the third (client) run => cargo run -- 3
#[tokio::main]
async fn main() {
    let node1 = Node::new("127.0.0.1".to_string(), 8888).expect("Failed to create Node1");

    let args: Vec<String> = env::args().collect();

    let server = &args[1];
    let mut server_bool: bool = false;
    if server.to_string() == "1" {
        server_bool = true;
        println!("Argument \"1\" passed, creating server...");
    }

    // Start the network listener in a separate task.
    if server_bool {
        let peer1 = Peer::new(&node1).await.unwrap();
        let listener_task = tokio::spawn(async move {
            let _ = peer1.create_listener().await;
        });

        let _ = tokio::try_join!(listener_task);
    } else {
        // To allow 2 connectors
        let mut port = 9999;
        let mut ip = "127.0.0.1".to_string();
        if server.to_string() == "2" {
            port = 9988;
            ip = "127.0.0.2".to_string();
        }
        let node2 = Node::new(ip, port).expect("Failed to create Node2");
        let peer2 = Peer::new(&node2).await.unwrap();
        // Start the network connector in a separate task.
        let connector_task = tokio::spawn(async move {
            let _ = peer2.create_sender(&node1).await;
        });

        let _ = tokio::try_join!(connector_task);

    }
}
