use std::process::exit;
use crate::kademlia::node::Node;
use crate::p2p::peer::Peer;
pub fn connect() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "listen" => {
                let new_node = Node::new("127.0.0.17".to_string(), 9988);
                if new_node.is_none() {
                    exit(1); // Terminate if node is invalid
                }
                let peer = Peer::new(new_node.unwrap());
                peer.start_listener();
            },
            "send" => {
                let place_holder_node = Node::new("127.0.0.2".to_string(), 9999);
                if place_holder_node.is_none() {
                    exit(1); // Terminate if node is invalid
                }
                let peer = Peer::new(place_holder_node.unwrap()); // This address is not used for sending
                let peer_address = "127.0.0.1:7878";
                let messages = vec!["Hello, peer!", "How are you?", "BYE"];
                peer.connect_to_peer(peer_address, messages);
            },
            _ => println!("Unknown command."),
        }
    } else {
        println!("Usage: program [listen | send]");
    }
}