use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cmp::Ordering;

use crate::auxi;
#[doc(inline)]
use crate::kademlia::bucket::Bucket;
use crate::kademlia::bucket::K;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::kademlia::trust_score::TrustScore;

pub const MAX_BUCKETS: usize = ID_LEN; // Max amount of Buckets (AKA amount of sub-tries)
pub const B: f64 = 0.65;

/// ## Kbucket
#[derive(Debug, Clone)]
pub struct KBucket {
    pub id: Identifier, // Here we can use some kind of identification (later on check if it's really needed)
    pub buckets: Vec<Bucket>, // The list of nodes in this bucket
}
impl KBucket {
    /// # new
    /// Create a new [KBucket] object.
    ///
    /// #### Returns
    /// Returns a new [KBucket] instance.
    pub fn new (id: Identifier) -> Self {
        KBucket {
            id,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    /// # add
    /// Proxy for the [Bucket::add] function.
    /// Adds a node to the corresponding bucket.
    /// The bucket in which the node will be placed
    /// is calculated through the XOR distance from the
    /// KBucket origin node
    /// #### Returns
    /// If the bucket is full, return the top [Node] otherwise return [None].
    pub fn add (&mut self, node: &Node) -> Option<Node> {
        // Don't add myself
        if node.id == self.id {
            return None;
        }

        // Get the index with relation to the buckets
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, &node.id);
        self.buckets[index].add(node)
        // None => Means we added the node to the vector
        // Node => Bucket was full, but we need to check if the latest contacted node is up, if not substitute
    }

    /// # replace_node
    /// Proxy for the [Bucket::replace_node] function.
    /// Attempts to replace the top node with the one passed as argument.
    /// Keep in mind that after removing the top node, the new node is added to
    /// the last position. In kademlia the nodes are stored from the older node contacted to the most recent (top to bottom).
    pub fn replace_node(&mut self, node: &Node){
        // This function will be called if a certain bucket is full
        // but we received a packet from a node which would belong to that
        // bucket and the latest stored node of that particular bucket is down
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, &node.id);
        self.buckets[index].replace_node(node); // Remove the top node and push back the passed node
    }

    /// # send_back
    /// Proxy for the [Bucket::send_back] function.
    /// Sends the top node of the bucket to the last position
    pub fn send_back(&mut self, node: &Node) {
        // This function will send the element on the top of the list to the back
        // Shifting upwards 1 every other node
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, &node.id);
        self.buckets[index].send_back();
    }

    pub fn send_back_specific_node(&mut self, node: &Node) {
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, &node.id);
        self.buckets[index].send_back_specific_node(node.clone());
    }

    /// # get
    /// Attempts to get a node from the correct [Bucket] according to the passed [id](Identifier).
    ///
    /// #### Returns
    /// If successful, return the [Node] otherwise return [None].
    pub fn get (&self, id: &Identifier) -> Option<Node>{
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);
        let bucket = &self.buckets[index];

        for i in bucket.map.iter() {
            if i.0.id == *id {
                return Some(i.0.clone())
            }
        }

        return None
    }

    pub fn get_trust_score(&self, id: &Identifier) -> Option<TrustScore> {
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);
        let bucket = &self.buckets[index];

        for i in bucket.map.iter() {
            if i.0.id == *id {
                return Some(i.1.clone())
            }
        }

        return None
    }

    /// # remove
    /// Proxy for the [Bucket::remove] function.
    /// Attempts to remove a node from its bucket.
    ///
    pub fn remove (&mut self, id: &Identifier){
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);

        if !Self::get(self, id).is_none() {
            let _ = &self.buckets[index].remove(id.clone());
            return;
        }
    }

    /// # get_nodes_from_bucket
    /// Get all nodes from a specific Bucket
    ///
    /// #### Returns
    /// Attempts to return a [Vec] of [nodes](Node), if none are found, return [None].
    pub fn get_nodes_from_bucket (&self, index: usize) -> Option<Vec<Node>> {
        let bucket = &self.buckets[index];
        let mut return_bucket = Vec::new();
        for k in bucket.map.iter() {

            return_bucket.push(k.0.clone());
        }

        if return_bucket.is_empty() {
            return None;
        }
        return Some(return_bucket);
    }

    /// # get_n_closest_nodes
    /// Get the n closest nodes from the given node
    ///
    /// #### Returns
    /// Attempts to fetch at least `n` nodes (the closest to the passed [id](Identifier)). If none are found
    /// return [None].
    pub fn get_n_closest_nodes (&self, id: Identifier, n: usize) -> Option<Vec<Node>>{
        let mut closest_nodes: Vec<Node> = Vec::new();
        let given_node_index: i64 = (MAX_BUCKETS - auxi::xor_distance(&id, &self.id)) as i64;
        let mut index;
        let mut i = 0;
        let mut j = 1;

        // The overall concept is:
        // 0 1 2 3 ... 90 target 92 93 94 ... MAX_BUCKETS
        // The closest nodes will be all that are inside target, then all that are inside 90 and 92, then those inside 89 and 93 and so on
        // What this for loop does is simply iterate on a zigzag pattern until it fills the vector
        for _ in 0..MAX_BUCKETS {
            if given_node_index + i < MAX_BUCKETS as i64 {
                index = given_node_index + i;
                if let Some(nodes) = self.get_nodes_from_bucket(index as usize) {
                    for node in nodes {
                        if closest_nodes.len() < n {
                            closest_nodes.push(node);
                        } else {
                            return Some(closest_nodes);
                        }
                    }
                }
                i += 1;
            }
            if given_node_index - j >= 0 {
                index = given_node_index - j;
                if let Some(nodes) = self.get_nodes_from_bucket(index as usize) {
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
        // If at the end the vector is empty return None
        if closest_nodes.is_empty() {
            return None;
        }

        // Otherwise it most likely means we weren't able to fill the vector
        // So we return the nodes we were able to gather
        Some(closest_nodes)

    }

    pub fn reputation_penalty(&mut self, identifier: Identifier) {
        let given_node_index = (MAX_BUCKETS - auxi::xor_distance(&self.id, &identifier)) as usize;
        self.buckets[given_node_index].reputation_penalty(identifier);
    }

    pub fn reputation_reward(&mut self, identifier: Identifier) {
        let given_node_index = (MAX_BUCKETS - auxi::xor_distance(&self.id, &identifier)) as usize;
        self.buckets[given_node_index].good_reputation(identifier);
    }

    pub fn risk_penalty(&mut self, identifier: Identifier) {
        let given_node_index = (MAX_BUCKETS - auxi::xor_distance(&self.id, &identifier)) as usize;
        self.buckets[given_node_index].risk_penalty(identifier);
    }

    pub fn increment_interactions(&mut self, identifier: Identifier) {
        let given_node_index = (MAX_BUCKETS - auxi::xor_distance(&self.id, &identifier)) as usize;
        self.buckets[given_node_index].increment_interactions(identifier);
    }

    pub fn increment_lookups(&mut self, identifier: Identifier) {
        let given_node_index = (MAX_BUCKETS - auxi::xor_distance(&self.id, &identifier)) as usize;
        self.buckets[given_node_index].increment_lookups(identifier);
    }
    pub fn get_all_trust_scores(&mut self) -> Vec<(Node, TrustScore)> {
        let mut trust_scores: Vec<(Node, TrustScore)> = Vec::new();
        for i in 0..MAX_BUCKETS {
            for j in self.buckets[i].map.iter_mut() {
                trust_scores.push((j.0.clone(), j.1.clone()));
            }
        }
        trust_scores
    }

    pub fn sort_by_new_distance(&self, nodes: Vec<Node>) -> Vec<Node> {
        let mut temp: Vec<(Node, TrustScore)> = Vec::new();
        for i in nodes {
            let mut trust = self.get_trust_score(&i.id);
            if !trust.is_none() {
                trust = Some(TrustScore::new());
            }
            temp.push((i, trust.unwrap()))
        }
        // Sort the nodes based in new_distance
        temp.sort_by(|mut a, mut b| {
            let old_distance1 = MAX_BUCKETS - auxi::xor_distance(&self.id, &a.0.id);
            let old_distance2 = MAX_BUCKETS - auxi::xor_distance(&self.id, &b.0.id);
            let score1 = a.1.clone().get_score();
            let score2 = b.1.clone().get_score();
            let new_distance1 = old_distance1 as f64 * B + (1f64 - B) * (1f64 / score1);
            let new_distance2 = old_distance2 as f64 * B + (1f64 - B) * (1f64 / score2);
            new_distance2.partial_cmp(&new_distance1).unwrap_or(Ordering::Equal)
        });
        let mut res: Vec<Node> = Vec::new();
        for (i, entry) in temp.iter().enumerate() {
            // Only return the K nearest
            if i == K {
                break;
            }
            res.push(entry.0.clone());
        }

        return res;
    }

}


mod tests {
    use crate::auxi;
    use crate::kademlia::k_buckets::KBucket;
    use crate::kademlia::node::Node;

    #[test]
    fn test_get () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbuckets = KBucket::new(node1.clone().id);
        kbuckets.add(&node1.clone());
        assert_eq!(kbuckets.get(&node1.id).unwrap(), node1);
    }

    #[test]
    fn test_remove () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbuckets = KBucket::new(node1.clone().id);
        kbuckets.add(&node1.clone());
        kbuckets.remove(&node1.clone().id);
        assert_eq!(kbuckets.get(&node1.id).is_none(), true);
    }

    #[test]
    fn test_get_nodes_from_bucket () {
        let ip = "127.0.0.17".to_string();
        let node = Node::new(ip.clone(), 9988).unwrap();

        let mut kbucket = KBucket::new(node.clone().id);

        for i in 1..=255 {
            let ip = format!("127.{}.0.{}", i, i);
            let port = 8888 + i;
            let _ = kbucket.add(&Node::new(ip, port).unwrap());
        }

        // Add more nodes to a same bucket (bucket 9 has 2 entries which will be used to test get_n_closest_nodes)
        let new_node = Node::new(ip.clone(), 8888+ 7);
        let node2 = new_node.unwrap();
        kbucket.add(&node2);

        let res = kbucket.get_nodes_from_bucket(132);
        assert!(!res.is_none());

        assert_eq!(res.unwrap().len(), 3);
    }

    #[test]
    fn test_get_n_closest_nodes () {
        let ip = "127.0.0.1".to_string();
        let node = Node::new(ip.clone(), 8888);
        let node1 = node.unwrap();

        let mut kbucket = KBucket::new(node1.clone().id);

        for i in 1..=3 {
            let new_node = Node::new(ip.clone(), 8888+i);
            let node2 = new_node.unwrap();
            kbucket.add(&node2);
        }

        // Add more nodes
        let new_node = Node::new(ip.clone(), 8895);
        let node1 = new_node.unwrap();
        kbucket.add(&node1);

        let node2 = auxi::gen_id("127.0.0.17:9988".to_string());
        let res = kbucket.get_n_closest_nodes(node2.clone(), 3);
        let res_content = res.unwrap();

        assert_eq!(res_content.len(), 3)

    }

}


