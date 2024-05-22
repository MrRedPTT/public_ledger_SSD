#[doc(inline)]
use std::fmt;
use sha2::{Digest, Sha512};
use crate::marco::sha512hash::Sha512Hash;


#[derive(Debug, Clone, PartialEq)]
///##Transactions
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub miner_fee: f64,
}

impl Transaction {
    /// creates a new transaction
    ///
    /// miner_fee is the difference of amount_in and amoun_out 
    pub fn new(amount_in: f64, from: String, amount_out: f64, to: String ) -> Transaction {
        return Transaction {
            from,
            to,
            amount_in,
            amount_out,
            miner_fee : amount_in - amount_out,
        };
    }
}

impl Sha512Hash for Transaction {
    fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount_in.to_le_bytes());
        hasher.update(self.amount_out.to_le_bytes());
        hasher.update(self.miner_fee.to_le_bytes());

        let hash_result = hasher.finalize();

        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
        return hash_hex;
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Transaction {{ from: {}, to: {}, amount_in: {}, amount_out: {}, miner_fee: {} }}",
            self.from, self.to, self.amount_in, self.amount_out, self.miner_fee
        )
    }
}
