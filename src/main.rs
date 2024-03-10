extern crate core;
use std::{env};
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;
use crate::proto::Address;
use crate::proto::packet_sending_server::{PacketSending};



mod kademlia{
    pub mod test_network;
    pub mod kademlia;
    pub mod node;
    pub mod k_buckets;
    pub mod key;

    pub mod bucket;
    pub mod auxi;
}
pub mod ledger;

mod p2p{
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
        let mut rpc = Peer::new(&node1).await.unwrap();

        // Here we are going to add some random ips
        for i in 1..=20 {
            let ip = format!("127.0.0.{}", i);
            let port = 8888 + i;
            let mut kademlia_ref = &mut *rpc.kademlia.lock().unwrap();
            let _ = kademlia_ref.add_node(&Node::new(ip, port).unwrap());
        }

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
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, Node::gen_id("127.0.0.2".to_string(), 8890)).await;
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, Node::gen_id("127.0.0.4".to_string(), 8889)).await;
        let _ = peer.ping("127.0.0.1", 8888).await;
        println!("{:?}", peer.ping("127.0.0.1", 7777).await);

    }
}

fn test_fn_gen_address(id: Identifier, ip: String, port: u32) -> Option<Address> {
    Some(proto::Address {
        id: id.0.to_vec(),
        ip,
        port
    })
}
