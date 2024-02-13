use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha512, Digest}; // Ensure sha2 crate is in Cargo.toml

#[derive(Debug, Clone)]
pub struct Block {
    pub timestamp: u64,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
}

impl Block {
    pub fn new(data: String, prev_hash: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        // Use Sha512 to hash the concatenated string of data, timestamp, and prev_hash
        let mut hasher = Sha512::new();
        hasher.update(format!("{}{}{}", &data, timestamp, &prev_hash));
        let hash_result = hasher.finalize(); // Finalize the hash
        let hash = hash_result.iter()
                          .map(|byte| format!("{:02x}", byte))
                          .collect::<Vec<String>>()
                          .join(""); // Convert the hash to a hexadecimal string

        Block {
            timestamp,
            data,
            prev_hash,
            hash,
        }
    }
}