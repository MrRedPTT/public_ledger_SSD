use crate::ledger::block::Block;
use crate::p2p::peer::Peer;

impl Peer {
    pub(crate) fn can_add(&self, block: Block) -> Option<bool> {
        // If None then the block is invalid
        // If True then the block was consumed
        // If False there's at least 1 block missing
        return Some(true);
    }
}