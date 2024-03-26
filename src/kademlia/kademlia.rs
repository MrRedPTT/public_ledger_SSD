use std::collections::HashMap;

use sha3::Digest;

use crate::auxi;
use crate::kademlia;
use crate::kademlia::bucket::K;
use crate::kademlia::k_buckets::{KBucket, MAX_BUCKETS};
use crate::kademlia::node::Identifier;
#[doc(inline)]
use crate::kademlia::node::Node;

#[derive(Debug, Clone)]
/// ## Kademlia
pub struct Kademlia {
    // Struct holding the node's state, routing table, etc.
    node: Node,
    kbuckets: KBucket,
    map: HashMap<Identifier, String>
}

impl Kademlia {

    /// # new
    /// Create a new instance of [Kademlia]
    ///
    /// #### Returns
    /// Will return a new [Kademlia] instance
    pub fn new (node: Node) -> Self {
        Kademlia {
            node: node.clone(),
            kbuckets: KBucket::new(node.clone().id),
            map: HashMap::new(),
        }
    }

    /// # add_key
    /// This function will add a new ([Key](Identifier), [Value](String)) to the [Kademlia] map.
    pub fn add_key(&mut self, key: Identifier, value: String) {
        let _ = self.map.insert(key, value);
    }

    /// # get_value
    /// This function will try to fetch a given [Value](String) from the map, given the
    /// corresponding passed [Key](Identifier).
    ///
    /// #### Returns
    /// If successful, return a [&String](String), otherwise return [None].
    pub fn get_value(&self, key: Identifier) -> Option<&String> {
        self.map.get(&key)
    }

    /// # remove_key
    /// Remove a ([Key](Identifier), [Value](String)) pair from the hashmap
    ///
    /// #### Returns
    /// If successful, return [true] else, [false]
    pub fn remove_key (&mut self, key: Identifier) -> bool {
        let _ = self.map.remove(&key);
        if !Self::get_value(&self, key).is_none() {
            return false;
        }
        return true;
    }

    /// # is_closest
    /// This function will check if the [self.node](Kademlia) is the closest to a given [Key](Identifier)
    ///
    /// #### Returns
    /// If true, return [None] otherwise, return, up to [K], nearest nodes in a [Vec<Node>](Vec) as an Option.
    pub fn is_closest(&self, key: &Identifier) -> Option<Vec<Node>>{
        // Remember that the following function returns the amount of 0's to the left
        // Meaning the higher the amount, the closest the 2 ids are
        let own_distance = MAX_BUCKETS - auxi::xor_distance(&self.node.id, key);
        let nodes = &self.kbuckets.get_n_closest_nodes(key.clone(), K);
        // If no nodes stored in the bucket with the closest nodes
        // Return None
        if nodes.is_none() {
            return None;
        }

        // For each node collected, check if any of them is closer to than ourselfs
        // If any is, return them, otherwise return None
        for i in <Option<Vec<Node>> as Clone>::clone(&nodes).unwrap() {
            if (MAX_BUCKETS - auxi::xor_distance(&i.id, key)) < own_distance {
                return nodes.clone();
            }
        }

        return None;
    }

    /// # add_node
    /// Proxy for the [KBucket::add] function.
    /// Attempts to add a [Node] to its corresponding [Bucket](kademlia::bucket)
    ///
    /// ### Returns
    /// If the bucket is full, return the top [Node] otherwise return [None].
    pub fn add_node (&mut self, node: &Node) -> Option<Node> {
        self.kbuckets.add(node)
    }

    /// # replace_node
    /// Proxy for the [KBucket::replace_node] function.
    /// Attempts to replace the top node with the one passed as argument.
    /// The correct [Bucket](kademlia::bucket) is calculated by this function.
    pub fn replace_node(&mut self, node: &Node){
        self.kbuckets.replace_node(node)
    }

    /// # send_back
    /// Proxy for the [KBucket::send_back] function.
    /// Attempts to send the top node from the [Bucket](kademlia::bucket) to the back of the
    /// [Vec](Vec), shifting every other node upwards.
    /// The correct [Bucket](kademlia::bucket) is calculated by this function.
    pub fn send_back(&mut self, node: &Node) {
        self.kbuckets.send_back(node);
    }


    /// # send_back_specific_node
    /// This function will take a node at an arbitrary position and move it to the back
    /// of the list
    /// This is needed in cases where the user contacts a node which is not currently at the top
    /// of the bucket, in which case [Kademlia::send_back] would be used.
    pub fn send_back_specific_node(&mut self, node: &Node){self.kbuckets.send_back_specific_node(node);}

    /// # get_node
    /// Proxy for the [KBucket::get] function.
    /// Attempts to fetch a node by a given [id](Identifier).
    ///
    /// #### Returns
    /// If the node exists return [Node] as an option, otherwise return [None].
    pub fn get_node (&self, id: Identifier) -> Option<Node> {
        return self.kbuckets.get(&id);
    }

    /// # get_k_nearest_to_node
    /// Proxy for the [KBucket::get_n_closest_nodes] function.
    /// Attempts to fetch the k nearest nodes to a given [id](Identifier).
    ///
    /// #### Returns
    /// Will return up K nearest nodes to the given node. If no node was found return [None].
    pub fn get_k_nearest_to_node(&self, id: Identifier) -> Option<Vec<Node>> {
        self.kbuckets.get_n_closest_nodes(id, K)
    }

    /// # remove_node
    /// Attempts to remove a node from the [KBucket]
    ///
    /// #### Returns
    /// If the node is still present in the [KBucket] return [false], otherwise return [true]
    pub fn remove_node (&mut self, id: Identifier) -> bool {
        self.kbuckets.remove(&id);
        return Self::get_node(self, id).is_none();
    }

}

#[cfg(test)]
mod tests {
    use crate::auxi;
    use crate::kademlia::kademlia::Kademlia;
    use crate::kademlia::node::Node;

    #[test]
    fn test_get_key() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        assert!(!node.is_none());
        let mut kademlia = Kademlia::new(node.unwrap());

        kademlia.add_key(auxi::gen_id("Some Key".to_string()), "Some Value".to_string());

        assert_eq!(*kademlia.get_value(auxi::gen_id("Some Key".to_string())).unwrap(), "Some Value".to_string())

    }

    #[test]
    fn test_remove_key() {
        let ip = "127.0.0.1".to_string();
        let port = 8888;
        let node = Node::new(ip.clone(), 8888);
        let mut kademlia = Kademlia::new(node.unwrap());

        kademlia.add_key(auxi::gen_id("Some Key".to_string()), "Some Value".to_string());
        kademlia.remove_key(auxi::gen_id("Some Key".to_string()));

        assert_eq!(kademlia.get_value(auxi::gen_id("Some Key".to_string())).is_none(), true)
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