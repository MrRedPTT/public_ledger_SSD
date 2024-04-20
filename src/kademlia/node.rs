#[doc(inline)]
use core::fmt;
use std::net::IpAddr;

use crate::auxi;

pub const ID_LEN: usize = 256; // Size in bits of SHA3_256 output (This is the hashing algorithm defined in Kademlia's documentation)
/// Identifier Type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct  Identifier(pub [u8; ID_LEN]); // hash of the ip:port of the node (can be changed later on to use the private certificates of the node)


impl Identifier {

    pub fn new(id: [u8; ID_LEN]) -> Self {
        Self(id)
    }

}

/// ## Node
#[derive(Debug, Clone, Eq, Hash)]
pub struct Node {
    pub id: Identifier, // Assuming Identifier is represented as a fixed-size array of 160 bytes
    pub ip: String,
    pub port: u32,
}

// Implement Display for Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let id_str = auxi::vec_u8_to_string(self.clone().id);
        // Customize the formatting of Node here
        write!(f, "Node {{ id:{:?} (SHA3_256: {}), ip: {}, port: {} }}", id_str, auxi::convert_node_id_to_string(&self.id), self.ip, self.port)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.port == other.port && self.port == other.port
    }
}

impl Node {
    /// Create a new instance of a Node
    pub fn new(ip: String, port: u32) -> Option<Self> {

        match ip.parse::<IpAddr>(){
            Ok(ip) => {
                if port > 65535 {
                    eprintln!("{} is an invalid port, try a value between 0 - 65535", port);
                    return None; // Return none if port is invalid
                }
                let id = auxi::gen_id(format!("{}:{}", ip, port).to_string());

                Some(Node {
                    id,
                    ip: ip.to_string(),
                    port
                })
            }
            Err(e) => {
                eprintln!("An error has occurred: {}", e);
                None
            }
        }
    }

    pub fn get_addr(self) -> String {
        format!("{}:{}", self.ip, self.port).to_string()
    }
}