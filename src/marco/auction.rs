#[doc(inline)]
use std::fmt;
use sha2::{Digest, Sha512};
use crate::marco::sha512hash::Sha512Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct Auction {
    auction_id : i64,
    seller_id: String,
    amount: f64,
}

impl Auction{} 

impl Sha512Hash for Auction {
    fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.auction_id.to_be_bytes());
        hasher.update(self.seller_id.as_bytes());
        hasher.update(self.amount.to_le_bytes());

        let hash_result = hasher.finalize();

        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
        return hash_hex;
    }
}

// Implementing the Display trait for the Auction struct
impl fmt::Display for Auction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Auction {{ auction_id: {}, seller_id: {}, amount: {} }}",
            self.auction_id, self.seller_id, self.amount)
    }
}
