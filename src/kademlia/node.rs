use core::fmt;
use std::net::SocketAddr;
use crate::kademlia::auxi;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Vec<u8>, // Tipically the node is represented by 160 bit Uid,
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
impl Node {
    pub fn new(id: Vec<u8>, address: SocketAddr) -> Self {
        Node {
            id,
            address,
        }
    }
}
