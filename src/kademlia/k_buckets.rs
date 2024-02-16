use crate::kademlia::node::Node;

#[derive(Debug, Clone)]
pub struct KBucket {
    nodes: Vec<Node>, // The list of nodes in this bucket
}

impl KBucket {
    pub fn new() -> Self {
        KBucket {
            nodes: Vec::new(),
        }
    }

    // We need methods to add and remove nodes from the kbucket
}
