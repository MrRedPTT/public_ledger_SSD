use core::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Vec<u8>, // Tipically the node is represented by 160 bit Uid,
    // using this we don't have to strictly adhere to the 160 bits, we can use more or less.
    pub address: SocketAddr, // IP address or hostname
}

// Implement Display for Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let byte_vector: Vec<u8> = self.id.to_vec();
        // Transform the Vec<u8> into a String
        let string_result: String = byte_vector.iter().map(|&byte| byte as char).collect();

        // Customize the formatting of Node here
        write!(f, "Node {{ id: {}, address: {} }}", string_result, self.address)
    }
}
impl Node {
    pub fn new(id: Vec<u8>, address: SocketAddr) -> Self {
        Node {
            id,
            address,
        }
    }


    pub fn get_node_distance(id1: &Vec<u8>, id2: &Vec<u8>)  -> Result<Vec<u8>, &'static str> {
        //We need to ensure the Ids are of the same length or else it doesn't work
        if id1.len() != id2.len() {
            return Err("ID vectors must be of the same length.");
        }

        let distance = id1.iter()// Creates an Iterator
            .zip(id2.iter()) // Creates a tuple with values from id1 and id2
            .map(|(&byte1, &byte2)| byte1 ^ byte2)// Applies the XOR
            .collect(); // Basically a return

        Ok(distance)
    }
}
