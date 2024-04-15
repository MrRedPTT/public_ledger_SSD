use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;

pub trait Observer: Send {
    fn on_block_received(&mut self, block: &Block) -> bool;
    fn on_transaction_received(&mut self, transaction: &Transaction) -> bool;
}

pub struct BlockchainEventSystem {
    observers: Vec<Arc<Mutex<dyn Observer>>>
}


impl BlockchainEventSystem {
    pub fn new() -> BlockchainEventSystem {
        BlockchainEventSystem {
            observers: vec![],
        }
    }

    pub fn notify_block_received(&self, block: &Block) -> bool {
        let mut flag = false;
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            if observer.on_block_received(&block) {
                flag = true;
            }
        }
        return flag; // if any of the subjects contains the block, return true (Most of the times we will only have 1 observer but this helps with possible future changes)
    }

    pub fn notify_transaction_received(&self, transaction: &Transaction) -> bool{
        let mut flag = false;
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            if observer.on_transaction_received(&transaction) {
                flag = true;
            }
        }
        return flag;
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn Observer>>) {
        self.observers.push(observer);
    }
}
impl Debug for BlockchainEventSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BlockchainEventSystem {{ observers: [")?;
        for (i, observer) in self.observers.iter().enumerate() {
            write!(f, "{}", i)?;
            if i < self.observers.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "] }}")
    }
}
