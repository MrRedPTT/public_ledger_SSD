use crate::kademlia::bucket::Bucket;
use crate::kademlia::node::Node;
use crate::kademlia::aux;

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
        let leading_zeros = Self::xor_distance(&self.id, &node.id);
        println!("DEBUG IN KBUCKETS::add: Leading Zeros: {}", leading_zeros);
        let index = MAX_BUCKETS - Self::xor_distance(&self.id, &node.id);
        match self.buckets[index].add(node).is_none() {
            true => true,
            false => false,
        }
    }

    pub fn get (&self, id: &Vec<u8>) -> Option<Node>{
        let index = MAX_BUCKETS - Self::xor_distance(&self.id, id);
        let bucket = &self.buckets[index];

        bucket.map.get(&aux::convert_node_id_to_string(id)).map(|socket_addr| Node {
            id: (*id).clone(),
            address: *socket_addr,
        })
    }

    // Applies XOR to each element of the Vector
    // and returns the XORed elements inside another Vector
    pub fn xor_distance(id1: &Vec<u8>, id2: &Vec<u8>) -> usize {
        println!("DEBUG IN KBUCKETS::xor_distance -> Size of vector1: {}, vector2: {}", id1.len(), id2.len());
        let res: Vec<u8> = id1
            .iter()
            .zip(id2.iter())
            .map(|(&x1, &x2)| x1 ^ x2)
            .collect();

        return Self::get_leading(&res.to_vec()) as usize;
    }

    // Return the number of leading zeros (aka number of bits different between the two nodes)
    fn get_leading(v: &[u8]) -> u32 {
        let n_zeroes = match v.iter().position(|&x| x != 0) {
            Some(n) => n,
            None => return 8*v.len() as u32,
        };

        v.get(n_zeroes).map_or(0, |x| x.leading_zeros()) + 8 * n_zeroes as u32
    }
}


