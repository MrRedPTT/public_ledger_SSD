#[doc(inline)]
use core::fmt;
use std::net::IpAddr;
use sha1::{Sha1, Digest};
use crate::kademlia::auxi;
use crate::kademlia::auxi::vec_u8_to_string;

/// Identifier Type
pub type Identifier = [u8; 160]; // hash of the ip:port of the node (can be changed later on to use the private certificates of the node)
pub const ID_LEN: usize = 160; // Size in bits of SHA1 output (This is the hashing algorithm defined in Kademlia's documentation)
/// ## Node
#[derive(Debug, Clone)]
pub struct Node {
    pub id: Identifier, // Tipically the node is represented by 160 bit Uid,
    // using this we don't have to strictly adhere to the 160 bits, we can use more or less.
    pub ip: String, // IP address or hostname
    pub port: u16,
}

// Implement Display for Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let id_str = vec_u8_to_string(self.id);
        // Customize the formatting of Node here
        write!(f, "Node {{ id:{:?} (SHA1: {}), ip: {}, port: {} }}", id_str, auxi::convert_node_id_to_string(&self.id), self.ip, self.port)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.port == other.port && self.port == other.port
    }
}

impl Node {
    /// Create a new instance of a Node
    pub fn new(ip: String, port: u16) -> Option<Self> {

        match ip.parse::<IpAddr>(){
            Ok(ip) => {
                if port > 65535 || port < 0 {
                    eprintln!("{} is an invalid port, try a value between 0 - 65535", port);
                    return None; // Return none if port is invalid
                }
                let id = Self::gen_id(ip.clone().to_string(), port);
                println!("DEBUG AT NODE::NEW => size of id: {} with id: {:?}", id.len(), id.clone());

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

    pub fn gen_id (ip: String, port: u16) -> [u8; 160] {
        let input = format!("{}:{}", ip, port.to_string());
        let mut hasher = Sha1::new();
        hasher.update(input.as_bytes());

        // we know that the hash output is going to be 256 bits = 32 bytes
        let result = hasher.finalize();

        let mut hash = [0; ID_LEN];
        let mut bin_str = "".to_string();
        for b in result {
            bin_str += &format!("{:0>8b}", b); // Force the representation to pad with 0's in case there aren't 8 bits
        }

        let mut i = 0;
        assert_eq!(bin_str.len(), 160); // assert thar we indeed have 160 bits
        for b in bin_str.chars() {
            hash[i] = b.to_digit(2).unwrap() as u8;
            i += 1;
        }
        hash
    }
}

