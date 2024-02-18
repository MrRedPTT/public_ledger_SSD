#[doc(inline)]
use crate::kademlia::node::Node;
use std::collections::HashMap;

use sha2::{Digest};
use sha2::digest::typenum::private::IsLessOrEqualPrivate;
use crate::kademlia::k_buckets::KBucket;

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
    pub fn add_node (&mut self, node: Node) -> bool {
        self.kbuckets.add(node)
    }

    /// Get the node for the given id
    pub fn get_node (&self, id: Vec<u8>) -> Option<Node> {
        return self.kbuckets.get(&id);
    }

    /// Remove a node by ID from the kbuckets
    pub fn remove_node (&mut self, id: Vec<u8>) -> bool {
        self.kbuckets.remove(&id);
        return Self::get_node(self, id).is_none();
    }

}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, SocketAddr};
    use crate::kademlia::kademlia::Kademlia;
    use crate::kademlia::node::Node;

    #[test]
    fn test_get_key() {
        let ip = "127.0.0.1";
        let port = 8888;
        let node = Node::new(vec![0; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), 8888));
        let mut kademlia = Kademlia::new(node);

        kademlia.add_key("Some Key".to_string(), "Some Value".to_string());

        assert_eq!(*kademlia.get_value("Some Key".to_string()).unwrap(), "Some Value".to_string())

    }

    #[test]
    fn test_remove_key() {
        let ip = "127.0.0.1";
        let port = 8888;
        let node = Node::new(vec![0; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), 8888));
        let mut kademlia = Kademlia::new(node);

        kademlia.add_key("Some Key".to_string(), "Some Value".to_string());
        kademlia.remove_key("Some Key".to_string());

        assert_eq!(kademlia.get_value("Some Key".to_string()).is_none(), true)
    }

    #[test]
    fn test_get_node() {
        let ip = "127.0.0.1";
        let port = 8888;
        let node = Node::new(vec![0; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), 8888));
        let mut kademlia = Kademlia::new(node);

        let new_node = Node::new(vec![1; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), port +1 ));

        kademlia.add_node(new_node.clone());

        assert_eq!(kademlia.get_node(new_node.clone().id).unwrap(), new_node)
    }

    #[test]
    fn test_remove_node() {
        let ip = "127.0.0.1";
        let port = 8888;
        let node = Node::new(vec![0; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), 8888));
        let mut kademlia = Kademlia::new(node);

        let new_node = Node::new(vec![1; 64], SocketAddr::new(ip.parse::<IpAddr>().unwrap(), port +1 ));

        kademlia.add_node(new_node.clone());
        kademlia.remove_node(new_node.clone().id);
        assert_eq!(kademlia.get_node(new_node.clone().id).is_none(), true)
    }
}