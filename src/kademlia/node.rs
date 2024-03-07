#[doc(inline)]
use core::fmt;
use std::net::IpAddr;
use sha3::{Digest, Sha3_256};
use crate::kademlia::auxi;
use crate::kademlia::auxi::vec_u8_to_string;


pub const ID_LEN: usize = 256; // Size in bits of SHA3_256 output (This is the hashing algorithm defined in Kademlia's documentation)
/// Identifier Type
pub type Identifier = [u8; ID_LEN]; // hash of the ip:port of the node (can be changed later on to use the private certificates of the node)
/// ## Node
#[derive(Debug, Clone)]
pub struct Node {
    pub id: Identifier, // Typically the node is represented by 160 bit Uid,
    // using this we don't have to strictly adhere to the 160 bits, we can use more or less.
    pub ip: String, // IP address or hostname
    pub port: u16,
}

// Implement Display for Node
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let id_str = vec_u8_to_string(self.id);
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

    pub fn gen_id (ip: String, port: u16) -> Identifier {
        let input = format!("{}:{}", ip, port.to_string());
        let mut hasher = Sha3_256::new();
        hasher.update(input.as_bytes());

        // we know that the hash output is going to be 256 bits = 32 bytes
        let result = hasher.finalize();

        let mut hash = [0; ID_LEN];
        let mut bin_str = "".to_string();
        for b in result {
            bin_str += &format!("{:0>8b}", b); // Force the representation to pad with 0's in case there aren't 8 bits
        }

        let mut i = 0;
        assert_eq!(bin_str.len(), ID_LEN); // assert thar we indeed have 160 bits
        for b in bin_str.chars() {
            hash[i] = b.to_digit(2).unwrap() as u8;
            i += 1;
        }
        hash
    }
}

mod test {
    use crate::kademlia::node::Node;

    #[test]
    fn test_gen_id(){
        let ip = "TestStringForTheSHA3_256HashingAlgorith".to_string();
        let port = 1;
        let res = Node::gen_id(ip, port);

        let mut bin_str = "".to_string();
        for i in 0..res.len() {
            bin_str += &format!("{}", res[i]);
        }
        assert_eq!(bin_str, "1000111111100100100001111100111100111011001000111001110000111011011100001010111000101010100110101101110100010010100011010010010000100101110000110010001001001100011100111100100111001101000011000101001010110011100110011100010000001000111001100101100001110010");

    }
}