#[doc(inline)]
use crate::kademlia::bucket::Bucket;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::kademlia::auxi;

pub const MAX_BUCKETS: usize = ID_LEN; // Max amount of Buckets (AKA amount of sub-tries)

/// ## Kbucket
#[derive(Debug, Clone)]
pub struct KBucket {
    pub id: Identifier, // Here we can use some kind of identification (later on check if it's really needed)
    pub buckets: Vec<Bucket>, // The list of nodes in this bucket
}
impl KBucket {
    /// Create a new KBucket Object
    pub fn new (id: Identifier) -> Self {
        KBucket {
            id,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    // Add a node to it's corresponding bucket
    /// Adds a node to the corresponding bucket.
    /// The bucket in which the node will be place
    /// is calculated through the XOR distance from the
    /// KBucket origin node
    pub fn add (&mut self, node: Node) -> bool {
        // Get the index with relation to the buckets
        let leading_zeros = auxi::xor_distance(&self.id, &node.id);
        println!("DEBUG IN KBUCKETS::add: Leading Zeros: {}", leading_zeros);
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, &node.id);
        match self.buckets[index].add(node).is_none() {
            true => true,
            false => false,
        }
    }

    /// Get a node with a specific ID.
    /// If such node does not exist, return None
    pub fn get (&self, id: &Identifier) -> Option<Node>{
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);
        let bucket = &self.buckets[index];

        bucket.map.get(id).map(| node | Node {
            id: (*id).clone(),
            ip: node.clone().ip,
            port: node.port
        })
    }

    /// Remove a node from its bucket
    pub fn remove (&mut self, id: &Identifier){
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);

        if !Self::get(self, id).is_none() {
            let _ = &self.buckets[index].map.remove(id);
            return;
        }
        println!("DEBUG FROM KBUCKET::remove() -> Node: {} not found", auxi::vec_u8_to_string((*id).clone()));
    }

    /// Get all nodes from a specific Bucket
    pub fn get_nodes_from_bucket (&self, index: usize) -> Option<Vec<Node>> {
        let bucket = &self.buckets[index];
        let mut return_bucket = Vec::new();
        let mut node: Node;
        for (k, v) in bucket.map.iter() {
            node = Node {
                id: (*k).clone(),
                ip: (*v).clone().ip,
                port: (*v).clone().port
            };
            return_bucket.push(node);
        }

        if return_bucket.is_empty() {
            return None;
        }
        return Some(return_bucket);
    }

    /// Get the n closest nodes from the given node
    pub fn get_n_closest_nodes (&self, id: Identifier, n: usize) -> Option<Vec<Node>>{
        let mut closest_nodes: Vec<Node> = Vec::new();
        let given_node_index = MAX_BUCKETS - auxi::xor_distance(&id, &self.id);
        let mut index;
        let mut i = 0;
        let mut j = 1;

        // The overall concept is:
        // 0 1 2 3 ... 90 target 92 93 94 ... MAX_BUCKETS
        // The closest nodes will be all that are inside target, then all that are inside 90 and 92, then those inside 89 and 93 and so on
        // What this for loop does is simply iterate on a zigzag pattern until it fills the vector
        for _ in 0..MAX_BUCKETS {
            if given_node_index + i < MAX_BUCKETS {
                index = given_node_index + i;
                if let Some(nodes) = self.get_nodes_from_bucket(index) {
                    for node in nodes {
                        if closest_nodes.len() < n {
                            closest_nodes.push(node);
                        } else {
                            return Some(closest_nodes);
                        }
                    }
                }
                i += 1;
                if given_node_index - j >= 0 {
                    index = given_node_index - j;
                    if let Some(nodes) = self.get_nodes_from_bucket(index) {
                        for node in nodes {
                            if closest_nodes.len() < n {
                                closest_nodes.push(node);
                            } else {
                                return Some(closest_nodes);
                            }
                        }
                    }
                    j += 1;
                }
            }
        }
        // If at the end the vector is empty return None
        if closest_nodes.is_empty() {
            return None;
        }

        // Otherwise it most likely means we weren't able to fill the vector
        // So we return the nodes we were able to gather
        Some(closest_nodes)

    }

}


mod tests {
    use crate::kademlia::k_buckets::KBucket;
    use crate::kademlia::node::Node;

    #[test]
    fn test_get () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbuckets = KBucket::new(node1.clone().id);
        kbuckets.add(node1.clone());
        assert_eq!(kbuckets.get(&node1.id).unwrap(), node1);
    }

    #[test]
    fn test_remove () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbuckets = KBucket::new(node1.clone().id);
        kbuckets.add(node1.clone());
        kbuckets.remove(&node1.clone().id);
        assert_eq!(kbuckets.get(&node1.id).is_none(), true);
    }

    #[test]
    fn test_get_nodes_from_bucket () {
        let ip = "127.0.0.17".to_string();
        let node = Node::new(ip.clone(), 9988);
        let node1 = node.unwrap();

        let mut kbucket = KBucket::new(node1.clone().id);

        let ip2 = "127.0.0.1".to_string();
        for i in 1..=3 {
            let new_node = Node::new(ip2.clone(), 8888+i as u16);
            let node2 = new_node.unwrap();
            kbucket.add(node2);
        }

        // Add more nodes to a same bucket (bucket 9 has 2 entries which will be used to test get_n_closest_nodes)
        let new_node = Node::new(ip.clone(), 8888+ 7u16);
        let node2 = new_node.unwrap();
        kbucket.add(node2);

        let res = kbucket.get_nodes_from_bucket(175);
        assert!(!res.is_none());

        assert_eq!(res.unwrap().len(), 1);
    }

    #[test]
    fn test_get_n_closest_nodes () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbucket = KBucket::new(node1.clone().id);

        for i in 1..=3 {
            let new_node = Node::new(ip.clone(), 8888+i as u16);
            let node2 = new_node.unwrap();
            kbucket.add(node2);
        }

        // Add more nodes
        let new_node = Node::new(ip.clone(), 8895);
        let node1 = new_node.unwrap();
        kbucket.add(node1);

        let node2 = Node::gen_id("127.0.0.17".to_string(), 9988);
        let res = kbucket.get_n_closest_nodes(node2.clone(), 3);
        let res_content = res.unwrap();

        assert_eq!(res_content.len(), 3)

    }

}


