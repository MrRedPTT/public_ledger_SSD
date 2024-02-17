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