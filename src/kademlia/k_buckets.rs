use crate::kademlia::bucket::Bucket;

pub const MAX_BUCKETS: usize = 10; // Max amount of Buckets (AKA amount of sub-tries)

#[derive(Debug, Clone)]
pub struct KBucket {
    pub id: String, // Here we can use some kind of identification (later on check if it's really needed)
    pub buckets: Vec<Bucket>, // The list of nodes in this bucket
}

impl KBucket {
    pub fn new (&mut self, id: String) -> Self {
        KBucket {
            id,
            buckets: vec![Default::default(); MAX_BUCKETS],
        }
    }
}


