#[doc(inline)]
use crate::kademlia::node::Node;
use std::collections::HashMap;

use sha3::{Digest};
use crate::kademlia::bucket::K;
use crate::kademlia::k_buckets::KBucket;
use crate::kademlia::node::Identifier;

#[derive(Debug, Clone)]
/// ## Kademlia
pub struct Kademlia {
    // Struct holding the node's state, routing table, etc.
    node: Node,
    kbuckets: KBucket,
    map: HashMap<String, String>
}

impl Kademlia {

    /// Create a new instance of Kademlia
    pub fn new (node: Node) -> Self {
        Kademlia {
            node: node.clone(),
            kbuckets: KBucket::new(node.clone().id),
            map: HashMap::new(),
        }
    }

    /// Store a pair (key, value) inside our local kademlia HashMap
    pub fn add_key(&mut self, key: String, value: String) {
        let _ = self.map.insert(key, value);
    }

    /// Get the value of a Key (Later on will be implemented with P2P to search on other nodes)
    pub fn get_value(&self, key: String) -> Option<&String> {
        self.map.get(&key)
    }

    /// Remove a (key, value) pair from the hashmap
    /// return false if it failed to do so
    pub fn remove_key (&mut self, key: String) -> bool {
        let _ = self.map.remove(&key);
        if !Self::get_value(&self, key).is_none() {
            return false;
        }
        return true;
    }

    /// Add node to the kbuckets
    /// Return true on success
    pub fn add_node (&mut self, node: &Node) -> Option<Node> {
        self.kbuckets.add(node)
    }

    pub fn replace_node(&mut self, node: &Node){
        self.kbuckets.replace_node(node)
    }

    pub fn send_back(&mut self, node: &Node) {
        self.kbuckets.send_back(node);
    }

    /// Get the node for the given id
    pub fn get_node (&self, id: Identifier) -> Option<Node> {
        return self.kbuckets.get(&id);
    }

    pub fn get_k_nearest_to_node(&self, id: Identifier) -> Option<Vec<Node>> {
        self.kbuckets.get_n_closest_nodes(id, K)
    }

    /// Remove a node by ID from the kbuckets
    pub fn remove_node (&mut self, id: Identifier) -> bool {
        self.kbuckets.remove(&id);
        return Self::get_node(self, id).is_none();
    }

}

#[cfg(test)]
mod tests {
    use crate::kademlia::kademlia::Kademlia;
    use crate::kademlia::node::Node;

    #[test]
    fn test_get_key() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        assert!(!node.is_none());
        let mut kademlia = Kademlia::new(node.unwrap());

        kademlia.add_key("Some Key".to_string(), "Some Value".to_string());

        assert_eq!(*kademlia.get_value("Some Key".to_string()).unwrap(), "Some Value".to_string())

    }

    #[test]
    fn test_remove_key() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        let mut kademlia = Kademlia::new(node.unwrap());

        kademlia.add_key("Some Key".to_string(), "Some Value".to_string());
        kademlia.remove_key("Some Key".to_string());

        assert_eq!(kademlia.get_value("Some Key".to_string()).is_none(), true)
    }

    #[test]
    fn test_get_node() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        assert!(!node.is_none());
        let mut kademlia = Kademlia::new(node.unwrap());

        let new_node = &Node::new(ip.clone(), port +1 );
        assert!(!new_node.is_none());

        kademlia.add_node(new_node.as_ref().unwrap());

        // Best way I found to avoid moving variables
        assert_eq!(kademlia.get_node(<Option<Node> as Clone>::clone(&new_node).unwrap().id.clone()).unwrap(), <Option<Node> as Clone>::clone(&new_node).unwrap())
    }

    #[test]
    fn test_remove_node() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        assert!(!node.is_none());
        let mut kademlia = Kademlia::new(node.unwrap());

        let new_node = Node::new(ip.clone(), port +1);
        assert!(!new_node.is_none());

        kademlia.add_node(&new_node.clone().unwrap());
        kademlia.remove_node(new_node.clone().unwrap().id);
        assert_eq!(kademlia.get_node(new_node.unwrap().id.clone()).is_none(), true)
    }
}