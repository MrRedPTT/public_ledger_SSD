use crate::kademlia::bucket::Bucket;
use crate::kademlia::node::Node;
use crate::kademlia::auxi;

pub const MAX_BUCKETS: usize = 512; // Max amount of Buckets (AKA amount of sub-tries)

#[derive(Debug, Clone)]
pub struct KBucket {
    pub id: Vec<u8>, // Here we can use some kind of identification (later on check if it's really needed)
    pub buckets: Vec<Bucket>, // The list of nodes in this bucket
}
impl KBucket {
    pub fn new (id: Vec<u8>) -> Self {
        KBucket {
            id,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }

    // Add a node to it's corresponding bucket
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

    pub fn get (&self, id: &Vec<u8>) -> Option<Node>{
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);
        let bucket = &self.buckets[index];

        bucket.map.get(id).map(|socket_addr| Node {
            id: (*id).clone(),
            address: *socket_addr,
        })
    }

    pub fn remove (&mut self, id: &Vec<u8>){
        let index = MAX_BUCKETS - auxi::xor_distance(&self.id, id);

        if !Self::get(self, id).is_none() {
            let _ = &self.buckets[index].map.remove(id);
            return;
        }
        println!("DEBUG FROM KBUCKET::remove() -> Node: {} not found", auxi::vec_u8_to_string((*id).clone()));
    }

    pub fn get_nodes_from_bucket (&self, index: usize) -> Option<Vec<Node>> {
        let bucket = &self.buckets[index];
        let mut return_bucket = Vec::new();
        let mut node: Node;
        for (k, v) in bucket.map.iter() {
            node = Node {
                id: (*k).clone(),
                address: *v,
            };
            return_bucket.push(node);
        }

        if return_bucket.is_empty() {
            return None;
        }
        return Some(return_bucket);
    }

    pub fn get_n_closest_nodes (&self, id: Vec<u8>, n: usize) -> Option<Vec<Node>>{
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
                        if closest_nodes.len() < n{
                            closest_nodes.push(node);
                        } else {
                            return Some(closest_nodes);
                        }
                    }
                }
                i += 1;
                if index - j >= 0 {
                    index = given_node_index - j;
                    if let Some(nodes) = self.get_nodes_from_bucket(index) {
                        for node in nodes {
                            if closest_nodes.len() < n{
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


