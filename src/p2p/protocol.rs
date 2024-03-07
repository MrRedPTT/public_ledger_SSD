use std::io;
use crate::kademlia::key::Key;
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;


pub enum Request {
    Ping,
    Store(Key),
    FindValue(Key),
    FindNode(Identifier)
}

pub enum Response {
    Pong,
    ValueResult(String),
    NodeResult(Vec<Node>),
    ReRout(Vec<Node>)
}

pub struct Protocol {
    node: Node,
    peer: Peer
}


impl Protocol {

    pub async fn new(node: &Node) -> Result<Self, io::Error> {
        let peer = Peer::new(node).await?;

        Ok(Protocol {
            node: node.clone(),
            peer
        })
    }
}

