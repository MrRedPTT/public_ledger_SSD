#[doc(inline)]
use std::fmt;
use sha2::{Digest, Sha512};
use crate::marco::sha512hash::Sha512Hash;


#[derive(Debug, Clone, PartialEq)]
///##Winner Transaction
pub struct Winner{
    pub auction: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
}

impl Winner {
    /// creates a new Winner transaction
    pub fn new(auction: String, amount: f64, from: String, to: String ) -> Winner {
        return Winner{
            from,
            to,
            amount,
            auction
        };
    }
}

impl Sha512Hash for Winner{
    fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.auction.as_bytes());

        let hash_result = hasher.finalize();

        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
        return hash_hex;
    }
}

impl fmt::Display for Winner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Winner Transactin {{ auction:{}, from: {}, to: {}, amount: {} }}",
            self.auction, self.from, self.to, self.amount,
        )
    }
}
