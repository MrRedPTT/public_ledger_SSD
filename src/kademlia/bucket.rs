#[doc(inline)]
use std::collections::VecDeque;
use crate::kademlia::node::{Identifier, Node};

/// Defines the maximum amount of nodes allowed per bucket
pub const K: usize = 3; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
/// ## Bucket
pub struct Bucket {
    pub map: VecDeque<Node>,
}

impl Bucket {

    /// Create a new instance of a Bucket
    pub fn new() -> Self {
        Bucket {
            map: VecDeque::new(),
        }
    }

    /// Add a new node to the bucket
    pub fn add(&mut self, node: &Node) -> Option<Node> {
        return if self.map.len() <= K {
            self.map.push_back(node.clone()); // Add node to the back of the Vector
            None
        } else {
            let top_node = self.map.get(0);
            if top_node.is_none() {
                return None;
            }
            Some(top_node.unwrap().clone())
        }
    }

    pub fn replace_node(&mut self, node: &Node){
        self.map.pop_front();
        self.map.push_back(node.clone());
    }

    pub fn remove(&mut self, id: Identifier){
        let mut i = 0;
        for node in self.map.iter() {
            if node.id == id {
                self.map.remove(i);
                return;
            }
            i += 1;
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

        assert!(bucket.map.contains(&node));
    }
}