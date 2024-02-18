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
}
