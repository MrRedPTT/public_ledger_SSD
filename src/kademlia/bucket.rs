#[doc(inline)]
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::kademlia::node::{Identifier, Node};

/// Defines the maximum amount of nodes allowed per bucket
pub const K: usize = 3; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
/// ## Bucket
pub struct Bucket {
    pub map: HashMap<Identifier, SocketAddr>,
}

impl Bucket {

    /// Create a new instance of a Bucket
    pub fn new() -> Self {
        Bucket {
            map: HashMap::new(),
        }
    }

    /// Add a new node to the bucket
    pub fn add(&mut self, node: Node) -> Option<SocketAddr> {
        if self.map.len() <= K {
            self.map.insert(node.id, node.address) // If the same key was already present, return the old value
        } else {
            None // Return None if the max value of nodes inside the bucket was reached
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, SocketAddr};
    use crate::kademlia::bucket::Bucket;
    use crate::kademlia::node::Node;


    #[test]
    fn test_add () {
        let ip = "127.0.0.1".parse::<IpAddr>().unwrap();
        let node = Node::new(vec![0; 64], SocketAddr::new(ip, 8888));

        let mut bucket = Bucket::new();
        bucket.add(node.clone());

        assert_eq!(*bucket.map.get(&node.clone().id).unwrap(), SocketAddr::new(ip, 8888))
    }
}