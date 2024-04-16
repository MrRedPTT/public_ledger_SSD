use std::borrow::Borrow;
use std::borrow::BorrowMut;
#[doc(inline)]
use std::collections::VecDeque;

use crate::kademlia::node::{Identifier, Node};
use crate::kademlia::trust_score::TrustScore;

/// Defines the maximum number of nodes allowed per bucket
pub const K: usize = 20; // Max bucket size
#[derive(Clone, Debug, Default, PartialEq)]
/// ## Bucket
pub struct Bucket {
    pub map: VecDeque<(Node, TrustScore)>,
}

impl Bucket {

    /// # new
    /// Create a new instance of a [Bucket]
    ///
    /// #### Returns
    /// Returns a new instance of [Bucket].
    pub fn new() -> Self {
        Bucket {
            map: VecDeque::new(),
        }
    }

    /// # add
    /// Add a new node to the [Bucket]
    ///
    /// #### Returns
    /// If the bucket is full, return the top [Node] otherwise return [None].
    pub fn add(&mut self, node: &Node) -> Option<Node> {
        return if self.map.len() < K {
            for i in self.map.iter() {
                if i.0.id == node.id { // Sanity to check to make sure the node is not inside the bucket
                    return None;
                }
            }
            self.map.push_back((node.clone(), TrustScore::new())); // Add node to the back of the Vector
            None
        } else {
            let top_node = self.map.get(0);
            if top_node.is_none() {
                return None;
            }
            Some(top_node.unwrap().clone().0)
        }
    }

    /// # replace_node
    /// Attempts to replace the top node with the one passed as argument.
    /// Keep in mind that after removing the top node, the new node is added to
    /// the last position. In kademlia the nodes are stored from the older node contacted to the most recent (top to bottom).
    pub fn replace_node(&mut self, node: &Node){
        self.map.pop_front();
        self.map.push_back((node.clone(), TrustScore::new()));
    }

    /// # send_back
    /// Sends the top node of the bucket to the last position
    pub fn send_back(&mut self) {
        let top = self.map.pop_front();
        if !top.is_none(){
            self.map.push_back(top.unwrap());
        }
    }

    /// # send_back_specific_node
    /// In some instances, the user may send a request
    /// to a node which is not on top of the list
    /// This function moves a node in an arbitrary position
    /// to the back
    pub fn send_back_specific_node(&mut self, node: Node) {
        let mut count: usize = 0;
        let mut trust = TrustScore::new();
        let mut flag = false;
        for i in self.map.iter() {
            if i.0 == node.clone() {
                trust = i.1.clone();
                flag = true;
                break;
            }
            count += 1;
        }
        // Had to take this snippet out of the for because
        // doing it inside the for means that we get both a mutable and immutable
        // reference to self in the same instance (Something Rust does not allow)
        if flag {
            self.map.remove(count);
            self.map.push_back((node.clone(), trust));
        }
        return;
    }


    /// # remove
    /// Attempts to remove a node according to the [id](Identifier) passed.
    pub fn remove(&mut self, id: Identifier){
        let mut i = 0;
        for node in self.map.iter() {
            if node.0.id == id {
                self.map.remove(i);
                return;
            }
            i += 1;
        }
    }

    pub fn reputation_penalty(&mut self, identifier: Identifier) {
        for node in self.map.iter_mut() {
            if node.0.id == identifier {
                node.1.bad_reputation();
                return;
            }
        }
    }

    pub fn risk_penalty(&mut self, identifier: Identifier) {
        for node in self.map.iter_mut() {
            if node.0.id == identifier {
                node.1.bad_interaction();
                return;
            }
        }
    }

    pub fn increment_interactions(&mut self, identifier: Identifier) {
        println!("Got inside the Increment Interactions id is server1?: {}", identifier == Node::new("127.0.0.1".to_string(), 8888).unwrap().id.clone());
        for node in self.map.iter_mut() {
            if node.0.id == identifier {
                node.1.borrow_mut().new_interaction();
                println!("Current score: {}", node.1.borrow_mut().get_score());
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kademlia::bucket::Bucket;
    use crate::kademlia::node::Node;
    use crate::kademlia::trust_score::TrustScore;

    #[test]
    fn test_add () {
        let ip = "127.0.0.1".to_string();
        let maybe_node = Node::new(ip, 8888);
        assert!(!maybe_node.is_none());
        let node = maybe_node.unwrap();

        let mut bucket = Bucket::new();
        bucket.add(&node.clone());

        assert!(bucket.map.contains(&(node, TrustScore::new())));
    }
}