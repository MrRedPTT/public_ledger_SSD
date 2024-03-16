use sha3::{Digest, Sha3_256};
// Auxiliary functions
#[doc(inline)]
use crate::kademlia::node::{Identifier};
use crate::kademlia::node::ID_LEN;
use crate::{p2p, proto};
use crate::proto::Address;


/// Converts a node identifier (Vec<u8>) into a string, after hashing
pub fn convert_node_id_to_string (node_id: &Identifier) -> String{
    let mut hasher = Sha3_256::new();
    hasher.update(node_id.0);
    let hashed_node_id= hasher.finalize();
    let string = hashed_node_id.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    return string;
}

/// Converts an Identifier (`Vec<u8>`) into a String
pub fn vec_u8_to_string (v: Identifier) -> String {
    let mut string_result: String = "".parse().unwrap();
    for x in &v.0 {
        string_result += &*x.to_string();
    }
    return string_result;
}

// Applies XOR to each element of the Vector
// and returns the XORed elements inside another Vector
/// Calculates the distance between two Identifiers using xor
pub fn xor_distance(id1: &Identifier, id2: &Identifier) -> usize {
    let mut res: [u8; ID_LEN] = [0; ID_LEN];
    for i in 0..ID_LEN {
        res[i] = id1.0[i] ^ id2.0[i];
    }

    return get_leading(res) as usize;
}

// Return the number of leading zeros (aka number of bits different between the two nodes)
/// Gets the number of leading zeros, i.e., the number of consecutive zeros on the left
fn get_leading(v: [u8; ID_LEN]) -> u32 {
    let mut count = 0;
    for i in v {
        if i == 0 {
            count += 1;
        }
    }
    return count;
}

pub fn gen_id (str: String) -> Identifier {
    let mut hasher = Sha3_256::new();
    hasher.update(str.as_bytes());

    // we know that the hash output is going to be 256 bits = 32 bytes
    let result = hasher.finalize();

    let mut hash = [0; ID_LEN];
    let mut bin_str = "".to_string();
    for b in result {
        bin_str += &format!("{:0>8b}", b); // Force the representation to pad with 0's in case there aren't 8 bits
    }

    let mut i = 0;
    assert_eq!(bin_str.len(), ID_LEN); // assert that we indeed have ID_LEN bits
    for b in bin_str.chars() {
        hash[i] = b.to_digit(2).unwrap() as u8;
        i += 1;
    }
    Identifier::new(hash)
}

pub fn gen_address(id: Identifier, ip: String, port: u32) -> Option<Address> {
    Some(proto::Address {
        id: id.0.to_vec(),
        ip,
        port
    })
}

pub fn return_option<T>(arg: T) -> Option<T> {
    Some(arg)
}