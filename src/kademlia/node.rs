#[doc(inline)]
use core::fmt;
use std::net::SocketAddr;
use crate::kademlia::auxi;

/// Identifier Type
pub type Identifier = Vec<u8>;

/// ## Node
#[derive(Debug, Clone)]
pub struct Node {
    pub id: Identifier, // Tipically the node is represented by 160 bit Uid,
    // using this we don't have to strictly adhere to the 160 bits, we can use more or less.
    pub address: SocketAddr, // IP address or hostname
}

// Implement Display for Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {


        // Customize the formatting of Node here
        write!(f, "Node {{ id:{:?} (SHA512: {}), address: {} }}", &self.id, auxi::convert_node_id_to_string(&self.id), self.address)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.address == other.address
    }
}
impl Node {
    /// Create a new instance of a Node
    pub fn new(id: Identifier, address: SocketAddr) -> Self {
        Node {
            id,
            address,
        }
    }
}

