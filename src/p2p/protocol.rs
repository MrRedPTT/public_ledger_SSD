use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;

pub struct Protocol {
    pub node: Node,
    pub peer: Peer
}


pub enum REQUEST {
    PING, // Check if node is online
    STORE(String, String), // Instructs a Node to store a Value (Node is found using distance)
    FindValue(String), // Returns a Value or the k nearest nodes to the value
    FindNode(Identifier) // Returns the k nearest nodes from the target id

}

pub enum RESPONSE {
    PONG, // Answer PING
    ReturnValue(String), // Return the result of the lookup
    ReturnNodes(Vec<Node>), // Return the k nearest nodes to a target id (Response to FIND_NODE)
    ReRoute(Vec<Node>) // Return the k nearest nodes to a value (Response to FIND_VALUE in case the node does not contain the value)
}

impl Protocol {

    pub fn new (node: Node) -> Self {
        let temp = node.clone();
        let peer: Peer = Peer::new(temp);

        Protocol {
            node,
            peer
        }
    }

}