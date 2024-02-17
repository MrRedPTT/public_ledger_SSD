
use std::net::SocketAddr;
use crate::kademlia::kademlia::Kademlia;
use std::net::IpAddr;
use crate::kademlia::node::Node;
// Init file to test the Kademlia P2P layer
pub fn test() {
    let my_ip = "127.0.0.1";
    let my_node;
    match my_ip.parse::<IpAddr>() {
        Ok(ip) => {
            // Clones will be used throughout this code snippet to avoid moving the variable
            let id = vec![0; 8];
            my_node = Node::new(id, SocketAddr::new(ip, 8888));
            let clone_node = my_node.clone(); // Cloning the node so that we can use it in the println
            let mut kademlia = Kademlia::new(my_node);
            kademlia.store(clone_node.clone());

            let mut result: Option<(&String, &Node)> = kademlia.find_node(vec![0; 8]);
            if result.is_none() {
               println!("Node {} was not found", vec_u8_to_string(clone_node.clone().id))
            } else {
                println!("Node {} has a value of {}", vec_u8_to_string(clone_node.clone().id), result.unwrap().1.address);
            }

            if !kademlia.remove_node(vec![0; 8]) {
                println!("Failed to Remove node");
            }

            result = kademlia.find_node(vec![0; 8]);
            // Duplicate code but it's only here for testing purposes
            if result.is_none() {
                println!("Node {} was not found", vec_u8_to_string(clone_node.clone().id))
            } else {
                println!("Node {} has a value of {}", vec_u8_to_string(clone_node.clone().id), result.unwrap().1.address);
            }

        }
        Err(e) => {
            eprintln!("Error parsing IP address: {}", e);
        }
    }

}

pub fn vec_u8_to_string (v: Vec<u8>) -> String {
    let mut string_result: String = "".parse().unwrap();
    for x in &v {
        string_result += &*x.to_string();
    }
    return string_result;
}