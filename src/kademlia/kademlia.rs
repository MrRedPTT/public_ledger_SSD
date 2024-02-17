use crate::kademlia::node::Node;
use std::collections::HashMap;
use crate::kademlia::auxi;

use sha2::{Digest};


pub struct Kademlia {
    // Struct holding the node's state, routing table, etc.
    node: Node,
    routingtable: HashMap<String, Node>,
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
        let key = value.clone().id;
        let hex_key = auxi::convert_node_id_to_string(&key);
        self.routingtable.insert(hex_key, value);
    }

    // Here we assume that the Key on the hash table is the Hashed nodeid
    // we also assume that the user has access to the "unhashed" version of the nodeid
    pub fn find_node(&mut self, node_id: Vec<u8>) -> Option<(&String, &Node)> {
        let hex_string = auxi::convert_node_id_to_string(&node_id);
        return self.routingtable.get_key_value(&hex_string);
    }

    pub fn remove_node (&mut self, node_id: Vec<u8>) -> bool{
        let hex_string = auxi::convert_node_id_to_string(&node_id);
        self.routingtable.remove(&hex_string);
        return self.find_node(node_id).is_none()

    }

    // May not be implemented
    //pub fn find_value(&self, key: Vec<u8>) {
    //    // Implementation here
    //}
}
