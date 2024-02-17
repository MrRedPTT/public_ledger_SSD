
use std::net::SocketAddr;
use crate::kademlia::kademlia::Kademlia;
use std::net::IpAddr;
use sha2::{Digest};
use crate::kademlia::auxi;
use crate::kademlia::k_buckets::KBucket;
use crate::kademlia::node::Node;
// Init file to test the Kademlia P2P layer
pub fn test() {
    let my_ip = "127.0.0.1";
    let my_node;
    match my_ip.parse::<IpAddr>() {
        Ok(ip) => {
            // Clones will be used throughout this code snippet to avoid moving the variable
            let id = vec![0; 64];
            my_node = Node::new(id, SocketAddr::new(ip, 8888));
            let clone_node = my_node.clone(); // Cloning the node so that we can use it in the println
            let mut kademlia = Kademlia::new(my_node);
            kademlia.store(clone_node.clone());

            let mut result: Option<(&Vec<u8>, &Node)> = kademlia.find_node(vec![0; 64]);
            if result.is_none() {
               println!("Node {} was not found", auxi::vec_u8_to_string(clone_node.clone().id))
            } else {
                println!("Node {} has a value of {}", auxi::vec_u8_to_string(clone_node.clone().id), result.unwrap().1.address);
            }

            if !kademlia.remove_node(vec![0; 64]) {
                println!("Failed to Remove node");
            }

            result = kademlia.find_node(vec![0; 64]);
            // Duplicate code but it's only here for testing purposes
            if result.is_none() {
                println!("Node {} was not found", auxi::vec_u8_to_string(clone_node.clone().id))
            } else {
                println!("Node {} has a value of {}", auxi::vec_u8_to_string(clone_node.clone().id), result.unwrap().1.address);
            }

            let mut kbucket = KBucket::new(clone_node.clone().id);
            kbucket.add(clone_node.clone());
            let result2 = kbucket.get(&clone_node.clone().id);
            if !result2.is_none(){
                println!("Node1 from kbucket: {}", result2.unwrap());
            }
            let result3 = kbucket.get(&vec![0; 64]);
            if !result3.is_none(){
                println!("Node2 does not exist");
            }
            kbucket.remove(&clone_node.clone().id);
            if kbucket.get(&clone_node.clone().id).is_none() {
                println!("Node1 was removed");
            }
        }
        Err(e) => {
            eprintln!("Error parsing IP address: {}", e);
        }
    }

}