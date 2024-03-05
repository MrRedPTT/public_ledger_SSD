use sha1::{Digest, Sha1};
// Auxiliary functions
#[doc(inline)]
use crate::kademlia::node::{Identifier};

/// Converts a node identifier (Vec<u8>) into a string, after hashing
pub fn convert_node_id_to_string (node_id: &Identifier) -> String{
    let mut hasher = Sha1::new();
    hasher.update(node_id);
    let hashed_node_id= hasher.finalize();
    let string = hashed_node_id.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    return string;
}

/// Converts an Identifier (Vec<u8>) into a String
pub fn vec_u8_to_string (v: Identifier) -> String {
    let mut string_result: String = "".parse().unwrap();
    for x in &v {
        string_result += &*x.to_string();
    }
    return string_result;
}

// Applies XOR to each element of the Vector
// and returns the XORed elements inside another Vector
/// Calculates the distance between two Identifiers using xor
pub fn xor_distance(id1: &Identifier, id2: &Identifier) -> usize {
    println!("DEBUG IN KBUCKETS::xor_distance -> Size of vector1: {}, vector2: {}", id1.len(), id2.len());
    let mut res: [u8; 160] = [0; 160];
    for i in 0..160 {
        res[i] = id1[i] ^ id2[i];
    }

    return get_leading(res) as usize;
}

// Return the number of leading zeros (aka number of bits different between the two nodes)
/// Gets the number of leading zeros, i.e., the number of consecutive zeros on the left
fn get_leading(v: [u8; 160]) -> u32 {
    let mut count = 0;
    for i in v {
        if i == 0 {
            count += 1;
        }
    }
    return count;
}
