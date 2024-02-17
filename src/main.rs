mod ledger;

use crate::ledger::blockchain::Blockchain;

fn main() {
    let blockchain = Blockchain::new(true);

    //blockchain.add_block("miner2317".to_string());
    //blockchain.add_block("miner2318".to_string());

    println!("{:#?}", blockchain);
}
