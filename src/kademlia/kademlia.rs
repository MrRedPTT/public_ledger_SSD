use crate::kademlia::node::Node;
use std::collections::HashMap;

use sha2::{Digest};


pub struct Kademlia {
    // Struct holding the node's state, routing table, etc.
    node: Node,
    routingtable: HashMap<Vec<u8>, Node>,
}

impl Kademlia {

    pub fn new (node: Node) -> Self {
        Kademlia {
            node,
            routingtable: HashMap::new(),
        }
    }

    //pub fn ping(&self, node: &Node) {
    //    // Implementation here
    //}

    // Here we assume that we will only store nodes
    pub fn store(&mut self, value: Node) {
        let _ = self.routingtable.insert(value.clone().id, value);
    }

    // Here we assume that the Key on the hash table is the Hashed nodeid
    // we also assume that the user has access to the "unhashed" version of the nodeid
    pub fn find_node(&mut self, node_id: Vec<u8>) -> Option<(&Vec<u8>, &Node)> {
        return self.routingtable.get_key_value(&node_id);
    }

    pub fn remove_node (&mut self, node_id: Vec<u8>) -> bool {
        if self.find_node(node_id.clone()).is_none() {
            return true;
        }
        let _ = self.routingtable.remove(&node_id);
        return true;

    }


    // May not be implemented
    //pub fn find_value(&self, key: Vec<u8>) {
    //    // Implementation here
    //}
}
