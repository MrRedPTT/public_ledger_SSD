use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use tonic::async_trait;

use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;

#[async_trait]
pub trait NetworkObserver: Send + Sync + 'static{
    fn on_block_received(&mut self, block: &Block) -> bool;
   fn on_transaction_received(&mut self, transaction: &Transaction) -> bool;
}

#[async_trait]
pub trait BlockchainObserver: Send + Sync + 'static{
    async fn on_block_mined(&self, block: &Block);
    async fn on_transaction_created(&self, transaction: &Transaction);
}

pub struct BlockchainEventSystem {
    observers: Vec<Arc<RwLock<dyn NetworkObserver>>>
}

pub struct NetworkEventSystem {
    observers: Vec<Arc<RwLock<dyn BlockchainObserver>>>
}


impl BlockchainEventSystem {
    pub fn new() -> BlockchainEventSystem {
        BlockchainEventSystem {
            observers: vec![],
        }
    }

    pub fn notify_block_received(&self, block: &Block) -> bool {
        let mut flag = false;
        for wrapped_observer in self.observers.iter() {
            if wrapped_observer.write().unwrap().on_block_received(&block) {
                flag = true;
            }
        }
        return flag; // if any of the subjects contains the block, return true (Most of the times we will only have 1 observer but this helps with possible future changes)
    }

    pub fn notify_transaction_received(&self, transaction: &Transaction) -> bool{
        let mut flag = false;
        let mut DEBUG_count = 0;
        for wrapped_observer in self.observers.iter() {
            println!("Iter: {}", DEBUG_count);
            if wrapped_observer.write().unwrap().on_transaction_received(&transaction) {
                flag = true;
            }
            DEBUG_count += 1;
        }
        return flag;
    }

    pub fn add_observer(&mut self, observer: Arc<RwLock<dyn NetworkObserver>>) {
        self.observers.push(observer);
    }
}
impl Debug for BlockchainEventSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BlockchainEventSystem {{ observers: [")?;
        for (i, _) in self.observers.iter().enumerate() {
            write!(f, "{}", i)?;
            if i < self.observers.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "] }}")
    }
}

impl Debug for NetworkEventSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "BlockchainEventSystem {{ observers: [")?;
        for (i, _) in self.observers.iter().enumerate() {
            write!(f, "{}", i)?;
            if i < self.observers.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "] }}")
    }
}

impl NetworkEventSystem {
    pub fn new() -> NetworkEventSystem {
        NetworkEventSystem {
            observers: vec![],
        }
    }

    pub async fn notify_block_mined(&self, block: &Block){
        for wrapped_observer in self.observers.iter() {
            wrapped_observer.read().unwrap().on_block_mined(&block).await;
        }
    }

    pub async fn notify_transaction_created(&self, transaction: &Transaction){
        for wrapped_observer in self.observers.iter() {
            wrapped_observer.read().unwrap().on_transaction_created(&transaction).await;
        }
    }

    pub fn add_observer(&mut self, observer: Arc<RwLock<dyn BlockchainObserver>>) {
        self.observers.push(observer);
    }
}
