use std::collections::HashMap;
use std::net::SocketAddr;
use crate::kademlia::node::Node;

pub const K: usize = 3; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bucket {
    pub map: HashMap<Vec<u8>, SocketAddr>,
    pub limit: usize,
}

impl Bucket {
    pub fn add(&mut self, node: Node) -> Option<SocketAddr> {
        if self.map.len() <= K {
            self.map.insert(node.id, node.address) // If the same key was already present, return the old value
        } else {
            None // Return None if the max value of nodes inside the bucket was reached
        }
    }
}