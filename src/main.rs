extern crate core;
use std::{env};
use crate::kademlia::auxi;
use crate::kademlia::node::{ID_LEN, Node};
use crate::p2p::peer::Peer;
use crate::proto::packet_sending_server::{PacketSending};



pub mod kademlia{
    pub mod test_network;
    pub mod kademlia;
    pub mod node;
    pub mod k_buckets;

    pub mod bucket;
    pub mod auxi;
}
pub mod ledger;

pub mod p2p{
    pub mod peer;
    pub mod req_handler;

}

pub mod proto {
    tonic::include_proto!("rpcpacket");
}
#[tokio::main]
async fn main() {

    let node1 = Node::new("127.0.0.1".to_string(), 8888).expect("Failed to create Node1");
    let node2 = &Node::new("127.0.0.1".to_string(), 9999).expect("Failed to create Node2");

    let args: Vec<String> = env::args().collect();

    let server = &args[1];
    let mut server_bool: bool = false;
    if server.to_string() == "1" {
        server_bool = true;
        println!("Argument \"1\" passed, creating server...");
    }

    if server_bool {
        let rpc = Peer::new(&node1).await.unwrap();

        // Here we are going to add some random ips
        for i in 1..=255 {
            let ip = format!("127.0.0.{}", i);
            let port = 8888 + i;
            let kademlia_ref = &mut *rpc.kademlia.lock().unwrap();
            let _ = kademlia_ref.add_node(&Node::new(ip, port).unwrap());
        }

        // Add a key value to the hashmap to be looked up
        let _ = rpc.kademlia.lock().unwrap().add_key(auxi::gen_id("Some Key".to_string()), "Some Value".to_string());

        let shutdown_rx = rpc.init_server().await;
        println!("Test async");

        // In order to keep the server running we created a new thread which is the one "holding" the main thread
        // from terminating (given the following await)
        // This new thread is only listening for CTRL + C signals, and when it detects one, it attempts to send
        // a value through the oneshot channel which, if successful, will return the value to the receiver,
        // proceeding with the execution, but given that it's the last instruction, the program will terminate
        let _ = shutdown_rx.await;

    } else {
        let target_node = &node1;
        let peer = &Peer::new(node2).await.unwrap();

        let _ = peer.ping(target_node.ip.as_ref(), target_node.port).await;
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, auxi::gen_id("127.0.0.2:8890".to_string())).await;
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, auxi::gen_id("127.0.0.4:8889".to_string())).await;
        let _ = peer.ping("127.0.0.1", 8888).await;
        println!("{:?}", peer.ping("127.0.0.1", 7777).await);
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key".to_string())).await;
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;
        let _ = peer.store(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string()), "Some Value2".to_string()).await; // This returns a LocalStore
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;

        let mut key = auxi::gen_id(format!("{}:{}", "127.0.0.20", 8888 +20).to_string()); // Generate an id equal to another node

        // Flip the last bit
        if key.0[ID_LEN -2 ] == 0 {
            key.0[ID_LEN -2 ] = 1;
        } else {
            key.0[ID_LEN -2 ] = 0;
        }

        let _ = peer.store(target_node.ip.clone(), target_node.port, key, "Some Value2".to_string()).await;

    }
}

