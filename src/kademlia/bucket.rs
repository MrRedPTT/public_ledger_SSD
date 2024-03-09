#[doc(inline)]
use std::collections::HashMap;
use crate::kademlia::node::{Identifier, Node};

/// Defines the maximum amount of nodes allowed per bucket
pub const K: usize = 3; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
/// ## Bucket
pub struct Bucket {
    pub map: HashMap<Identifier, Node>,
}

impl Bucket {

    /// Create a new instance of a Bucket
    pub fn new() -> Self {
        Bucket {
            map: HashMap::new(),
        }
    }

    /// Add a new node to the bucket
    pub fn add(&mut self, node: &Node) -> Option<Node> {
        if self.map.len() <= K {
            self.map.insert(node.clone().id, node.clone()) // If the same key was already present, return the old value
        } else {
            None // Return None if the max value of nodes inside the bucket was reached
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kademlia::bucket::Bucket;
    use crate::kademlia::node::Node;


    #[test]
    fn test_add () {
        let ip = "127.0.0.1".to_string();
        let maybe_node = Node::new(ip, 8888);
        assert!(!maybe_node.is_none());
        let node = maybe_node.unwrap();

        let mut bucket = Bucket::new();
        bucket.add(&node.clone());

        assert_eq!(*bucket.map.get(&node.clone().id).unwrap(), node)
    }
}