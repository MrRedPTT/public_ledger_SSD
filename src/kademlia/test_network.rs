use crate::kademlia::kademlia::Kademlia;
use std::net::IpAddr;
use std::process::exit;
use crate::kademlia::auxi;
use crate::kademlia::k_buckets::{KBucket, MAX_BUCKETS};
use crate::kademlia::node::{ID_LEN, Identifier, Node};

// Init file to test the Kademlia P2P layer
pub async fn test() {
    let my_ip = "127.0.0.1";
    let my_node;
    match my_ip.parse::<IpAddr>() {
        Ok(ip) => {
            // Clones will be used throughout this code snippet to avoid moving the variable
            my_node = Node::new(ip.to_string(), 8888);
            if my_node.is_none() {
                exit(1); // Terminate if node is invalid
            }
            let clone_node = my_node.clone(); // Cloning the node so that we can use it in the println
            let mut kademlia = Kademlia::new(my_node.clone().unwrap());
            kademlia.add_node(clone_node.as_ref().unwrap());
            let mut result: Option<Node> = kademlia.get_node(auxi::gen_id(format!("{}:{}",my_node.clone().unwrap().ip, my_node.clone().unwrap().port).to_string()));
            if result.is_none() {
               println!("Node {} was not found", auxi::vec_u8_to_string(clone_node.clone().unwrap().id))
            } else {
                println!("Node {} has a value of ip: {}, port: {}", auxi::vec_u8_to_string(clone_node.clone().unwrap().id), result.clone().unwrap().ip, result.clone().unwrap().port);
            }

            if !kademlia.remove_node(auxi::gen_id(format!("{}:{}",my_node.clone().unwrap().ip, my_node.clone().unwrap().port).to_string())) {
                println!("Failed to Remove node");
            }

            result = kademlia.get_node(auxi::gen_id(format!("{}:{}",my_node.clone().unwrap().ip, my_node.clone().unwrap().port).to_string()));
            // Duplicate code but it's only here for testing purposes
            if result.is_none() {
                println!("Node {} was not found", auxi::vec_u8_to_string(clone_node.clone().unwrap().id))
            } else {
                println!("Node {} has a value of ip: {}, port: {}", auxi::vec_u8_to_string(clone_node.clone().unwrap().id), result.clone().unwrap().ip, result.clone().unwrap().port);
            }

            let mut kbucket = KBucket::new(clone_node.clone().unwrap().id);
            kbucket.add(&clone_node.clone().unwrap());
            let result2 = kbucket.get(&clone_node.clone().unwrap().id);
            if !result2.is_none(){
                println!("Node1 from kbucket: {}", result2.unwrap());
            }
            let result3 = kbucket.get(&auxi::gen_id(format!("{}:{}",my_node.clone().unwrap().ip, my_node.clone().unwrap().port).to_string()));
            if !result3.is_none(){
                println!("Node2 does not exist");
            }
            kbucket.remove(&clone_node.clone().unwrap().id);
            if kbucket.get(&clone_node.clone().unwrap().id).is_none() {
                println!("Node1 was removed");
            }

            for i in 1..=3 {
                let new_node = Node::new(ip.to_string(), 8888+i);
                if new_node.is_none() {
                    exit(1); // Terminate if node is invalid
                }
                kbucket.add(&new_node.unwrap());
            }

            // Add more nodes to a same bucket (bucket 9 has 2 entries which will be used to test get_n_closest_nodes)
            let new_node = Node::new(ip.to_string(), 8888+ 7);
            if new_node.is_none() {
                exit(1); // Terminate if node is invalid
            }
            kbucket.add(&new_node.unwrap());


            for i in 0..MAX_BUCKETS {
                let bucket_nodes = kbucket.get_nodes_from_bucket(i);
                if !bucket_nodes.is_none() {
                    println!("Nodes in bucket {}", i);
                    for n in bucket_nodes.unwrap() {
                        println!("{}", n);
                    }
                }
            }
            let fake_node = auxi::gen_id("127.0.0.17:9988".to_string());
            let closest = kbucket.get_n_closest_nodes(fake_node.clone(), 3);
            if !closest.is_none() {
                println!("Printing the {} closes nodes to {:?}, address: 127.0.0.17:9988", 3, fake_node);
                for node in closest.unwrap() {
                    println!("{} => {}", node, ID_LEN - auxi::xor_distance(&fake_node as &Identifier, &node.id as &Identifier));
                }
            }else {
                println!("No Nodes found");
            }

            // A simple test for the hash gen
            println!("Hash: {:?}", auxi::gen_id("192.45.121.871:12189".to_string()));


        }
        Err(e) => {
            eprintln!("Error parsing IP address: {}", e);
        }
    }

}