#[doc(inline)]
use std::fmt;

use sha2::{Digest, Sha512};

use crate::marco::sha512hash::Sha512Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct Auction {
    pub(crate) auction_id : i64,
    pub(crate) seller_id: String,
    pub(crate) amount: f64,
}

impl Auction{
    pub fn new( seller_id: String, amount: f64) -> Auction{
        Auction{
            auction_id: 1,
            seller_id,
            amount
        }
    }
} 

impl Sha512Hash for Auction {
    fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
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
        write!(f,"Auction {{seller_id: {}, amount: {} }}",
            self.seller_id, self.amount)
    }
}
