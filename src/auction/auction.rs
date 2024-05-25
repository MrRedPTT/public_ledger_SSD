use crate::auxi;
use crate::kademlia::node::Node;
use crate::p2p::peer::Peer;

pub struct Auction {
    pub client: Peer
}

impl Auction {
    pub async fn new() -> Self {
        let node = Node::new("127.0.0.1".to_string(), auxi::get_port().await as u32);
        let (client, server) = Peer::new(&node.unwrap(), false);
        let _ = server.init_server().await;
        client.boot().await;

        Auction {
            client
        }
    }

    pub async fn open_auction(&self) {
        todo!()
        // Acquire information from the stdin to populate the Marco object

        // This will broadcast the Marco through the network
        // self.client.send_marco(marco).await;
    }

    pub async fn place_bid(&self) {
        todo!()
        // Acquire information from the stdin to populate the Marco object

        // Once again, use the following call to broadcast the Marco
        // self.client.send_marco(marco).await;
    }

    pub async fn winner(&self) {
        todo!()
        // Acquire information from the stdin to populate the Marco object

        // Once again, use the following call to broadcast the Marco
        // self.client.send_marco(marco).await;
    }

    pub async fn search_auctions(&self) {
        todo!()
        // Iterate through the blockchain to collect all the open auctions
    }

}