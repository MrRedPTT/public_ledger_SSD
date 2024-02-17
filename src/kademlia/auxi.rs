// Auxiliary functions
use sha2::{Digest, Sha512};

pub fn convert_node_id_to_string (node_id: &Vec<u8>) -> String{
    let mut hasher = Sha512::new();
    hasher.update(node_id);
    let hashed_node_id= hasher.finalize();
    let string = hashed_node_id.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    return string;
}

pub fn vec_u8_to_string (v: Vec<u8>) -> String {
    let mut string_result: String = "".parse().unwrap();
    for x in &v {
        string_result += &*x.to_string();
    }
    return string_result;
}

// Applies XOR to each element of the Vector
// and returns the XORed elements inside another Vector
pub fn xor_distance(id1: &Vec<u8>, id2: &Vec<u8>) -> usize {
    println!("DEBUG IN KBUCKETS::xor_distance -> Size of vector1: {}, vector2: {}", id1.len(), id2.len());
    let res: Vec<u8> = id1
        .iter()
        .zip(id2.iter())
        .map(|(&x1, &x2)| x1 ^ x2)
        .collect();

    return get_leading(&res.to_vec()) as usize;
}

// Return the number of leading zeros (aka number of bits different between the two nodes)
fn get_leading(v: &[u8]) -> u32 {
    let n_zeroes = match v.iter().position(|&x| x != 0) {
        Some(n) => n,
        None => return 8*v.len() as u32,
    };

    v.get(n_zeroes).map_or(0, |x| x.leading_zeros()) + 8 * n_zeroes as u32
}