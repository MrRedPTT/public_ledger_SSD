use crate::kademlia::node::Node;
use std::collections::HashMap;

use sha2::{Sha512, Digest};


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
        let hex_key = Self::convert_node_id_to_string(key);
        self.routingtable.insert(hex_key, value);
    }

    // Here we assume that the Key on the hash table is the Hashed nodeid
    // we also assume that the user has access to the "unhashed" version of the nodeid
    pub fn find_node(&mut self, node_id: Vec<u8>) -> Option<(&String, &Node)> {
        let hex_string = Self::convert_node_id_to_string(node_id);
        return self.routingtable.get_key_value(&hex_string);
    }

    //pub fn find_value(&self, key: Vec<u8>) {
    //    // Implementation here
    //}

    pub fn convert_node_id_to_string (node_id: Vec<u8>) -> String{
        let mut hasher = Sha512::new();
        hasher.update(node_id);
        let hashed_node_id= hasher.finalize();
        let string = hashed_node_id.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        return string;
    }
}
