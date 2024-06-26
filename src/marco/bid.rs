#[doc(inline)]
use std::fmt;

use sha2::{Digest, Sha512};

use crate::marco::sha512hash::Sha512Hash;

#[derive(Debug, Clone, PartialEq)]
pub struct Bid {
    pub auction_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount: f64,
}

impl Bid{
    pub fn new(auction_id:String, buyer_id: String, seller_id: String, amount: f64) -> Bid{
        Bid {
            buyer_id,
            seller_id,
            amount,
            auction_id,
        }
    }

}

impl Sha512Hash for Bid {
    fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.auction_id.as_bytes());
        hasher.update(self.buyer_id.as_bytes());
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

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Bid {{ auction_id: {}, buyer_id: {}, seller_id: {}, amount: {} }}",
            self.auction_id, self.buyer_id, self.seller_id, self.amount
        )
    }
}
