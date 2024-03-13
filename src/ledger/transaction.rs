
use sha2::{Sha512, Digest};
#[derive(Debug, Clone)]

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

    /// returns the transaction in string format
    pub fn to_string(&self) -> String {
        return format!("{} from {}, {} to {} with {} fees",
                self.amount_in,
                self.from,
                self.amount_out,
                self.to,
                self.miner_fee);

    }

    pub fn to_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.to_string());
        let hash_result = hasher.finalize();

        let hash_hex = hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
        return hash_hex
    }
}
