use std::collections::HashMap;
use std::net::SocketAddr;
use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::Node;

pub const K: usize = 3; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<String, SocketAddr>,
    pub limit: usize,
}

impl Bucket {
    fn add(&mut self, node: Node) -> Option<SocketAddr> {
        if self.map.len() <= K {
            self.map.insert(Kademlia::convert_node_id_to_string(&node.id), node.address) // If the same key was already present, return the old value
        } else {
            None // Return None if the max value of nodes inside the bucket was reached
        }
    }
}