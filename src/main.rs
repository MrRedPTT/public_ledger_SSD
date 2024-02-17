mod block; // Import block
mod blockchain;  // Import blockChain

mod p2p{
    pub mod peer;
    pub mod connect;
}
fn main() {
    crate::p2p::connect::connect();
}
