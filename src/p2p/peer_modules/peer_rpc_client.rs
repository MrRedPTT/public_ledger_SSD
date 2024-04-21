use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::io;
use std::io::ErrorKind;

use crate::kademlia::node::{Identifier, Node};
use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::Peer;
use crate::p2p::private::broadcast_api::BroadCastReq;

#[derive(Clone)]
pub(crate) struct NodeNewDistance {
    pub node: Node,
    pub new_distance: f64
}

impl NodeNewDistance {
    pub fn new(node: Node, new_distance: f64) -> Self {
        NodeNewDistance {
            node,
            new_distance
        }
    }
}

impl Eq for NodeNewDistance {}

impl PartialEq<Self> for NodeNewDistance {
    fn eq(&self, other: &Self) -> bool {
        self.new_distance == other.new_distance
    }
}

impl PartialOrd<Self> for NodeNewDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.new_distance.partial_cmp(&other.new_distance)
    }
}

impl Ord for NodeNewDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        other.new_distance.partial_cmp(&self.new_distance).unwrap_or(Ordering::Equal)
    }
}

impl Peer {

    pub async fn send_block(&self, block: Block) {
        BroadCastReq::broadcast(self, None, Some(block), None, None).await;
    }

    pub async fn send_transaction(&self, transaction: Transaction) {
        BroadCastReq::broadcast(self, Some(transaction), None, None, None).await;
    }
    pub async fn find_node(&self, id: Identifier) -> Result<Node, io::Error>
    {
        let nodes = &mut self.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone()).unwrap_or(Vec::new());
        if nodes.len() == 0 {
            return Err(io::Error::new(ErrorKind::InvalidData, "No nodes found to communicate with"));
        }
        let mut reroute_table: HashMap<Node, Vec<Node>> = HashMap::new();
        let mut already_checked: Vec<Node> = Vec::new();
        // First iterate through our own nodes (according to old distance)
        let priority_queue: &mut BinaryHeap<NodeNewDistance> = &mut BinaryHeap::new();
        while !&nodes.is_empty(){
            let res = self.find_node_handler(id.clone(), self.get_batch(Some(nodes), None, 14), already_checked.borrow_mut(), reroute_table.borrow_mut()).await;

            match res {
                Ok(res_node) => {
                    if !res_node.is_none() {
                        self.kademlia.lock().unwrap().reputation_reward(res_node.clone().unwrap().1.id);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(list) => {
                    if !list.is_none() {
                        for i in list.clone().unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }

        while !priority_queue.is_empty() {
            let batch = self.get_batch(None, Some(priority_queue), 14);
            let res = self.find_node_handler(id.clone(), batch, already_checked.borrow_mut(), reroute_table.borrow_mut()).await;
            match res {
                Ok(res_node) => {
                    if !res_node.is_none() {
                        self.reward(reroute_table.borrow_mut(), &already_checked, res_node.clone().unwrap().1);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(e) => {
                    if !e.is_none() {
                        for i in e.unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }


        return Err(io::Error::new(ErrorKind::NotFound, "The node was not found"));
    }

    pub async fn find_value(&self, id: Identifier) -> Result<String, io::Error>
    {
        let nodes = &mut self.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone()).unwrap_or(Vec::new());
        if nodes.len() == 0 {
            return Err(io::Error::new(ErrorKind::InvalidData, "No nodes found to communicate with"));
        }
        let mut reroute_table: HashMap<Node, Vec<Node>> = HashMap::new();
        let mut already_checked: Vec<Node> = Vec::new();
        // First iterate through our own nodes (according to old distance)
        let priority_queue: &mut BinaryHeap<NodeNewDistance> = &mut BinaryHeap::new();
        while !&nodes.is_empty(){
            let res = self.find_value_handler(id.clone(), self.get_batch(Some(nodes), None, 14), already_checked.borrow_mut(), reroute_table.borrow_mut()).await;

            match res {
                Ok(res_node) => {
                    if !res_node.is_none() {
                        self.kademlia.lock().unwrap().reputation_reward(res_node.clone().unwrap().1.id);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(list) => {
                    if !list.is_none() {
                        for i in list.clone().unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }

        while !priority_queue.is_empty() {
            println!("New iter: Size of queue: {}", priority_queue.len());
            let batch = self.get_batch(None, Some(priority_queue), 14);
            let res = self.find_value_handler(id.clone(), batch, already_checked.borrow_mut(), reroute_table.borrow_mut()).await;
            match res {
                Ok(res_node) => {
                    if !res_node.is_none() {
                        self.reward(reroute_table.borrow_mut(), &already_checked, res_node.clone().unwrap().1);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(e) => {
                    if !e.is_none() {
                        for i in e.unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }


        return Err(io::Error::new(ErrorKind::NotFound, "The value was not found"));
    }


    /// Will try to get the block request and add it to the blockchain.
    /// Returns Ok(true) if the block (and possibly all dependent blocks before) were added.
    /// Returns Ok(false) if some of the dependent blocks were added but not all.
    /// Returns Err(e) if an error occured while trying to get the nodes.
    pub async fn get_block(&self, id: String) -> Result<bool, io::Error> {
        let mut block_trail: Vec<Block> = Vec::new();
        let res = self.get_block_auxi(id).await;
        let mut current_block: Block;
        match res {
            Err(e) => {return Err(e);},
            Ok(block) => {current_block = block;}
        }

        while !self.blockchain.lock().unwrap().can_add_block(current_block.clone()) {
            println!("A new block is needed with hash: {}", current_block.prev_hash.clone());
            let prev_block = self.get_block_auxi(current_block.prev_hash.clone()).await;
            match prev_block {
                Err(e) => {return Err(e);},
                Ok(bl) => {
                    /*
                    if !bl.valid {
                        return Err(some_error);
                    }
                     */
                    block_trail.push(bl.clone());
                    current_block = bl.clone();
                }
            }
        }

        for i in (0..block_trail.len()).rev() {
            if self.blockchain.lock().unwrap().can_add_block(block_trail[i].clone()) {
                self.blockchain.lock().unwrap().add_block(block_trail[i].clone());
            } else {
                return Ok(false);
            }
        }

        return Ok(true);
    }

    pub async fn get_block_auxi(&self, id: String) -> Result<Block, io::Error>
    {
        let nodes = &mut self.kademlia.lock().unwrap().get_k_nodes_new_distance().unwrap_or(Vec::new());
        if nodes.len() == 0 {
            return Err(io::Error::new(ErrorKind::InvalidData, "No nodes found to communicate with"));
        }
        let mut reroute_table: HashMap<Node, Vec<Node>> = HashMap::new();
        let mut already_checked: Vec<Node> = Vec::new();
        // First iterate through our own nodes (according to old distance)
        let priority_queue: &mut BinaryHeap<NodeNewDistance> = &mut BinaryHeap::new();
        while !&nodes.is_empty(){
            let res = self.get_block_handler(id.clone(), self.get_batch(Some(nodes), None, 14), already_checked.borrow_mut(), reroute_table.borrow_mut()).await;

            match res {
                Ok(res_node) => {
                    if !res_node.is_none() && res_node.clone().unwrap().0.hash == id {
                        self.kademlia.lock().unwrap().reputation_reward(res_node.clone().unwrap().1.id);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(list) => {
                    if !list.is_none() {
                        for i in list.clone().unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }

        while !priority_queue.is_empty() {
            let batch = self.get_batch(None, Some(priority_queue), 14);
            let res = self.get_block_handler(id.clone(), batch, already_checked.borrow_mut(), reroute_table.borrow_mut()).await;
            match res {
                Ok(res_node) => {
                    if !res_node.is_none() && res_node.clone().unwrap().0.hash == id {
                        self.reward(reroute_table.borrow_mut(), &already_checked, res_node.clone().unwrap().1);
                        return Ok(res_node.unwrap().0);
                    }
                }
                Err(e) => {
                    if !e.is_none() {
                        for i in e.unwrap() {
                            priority_queue.push(NodeNewDistance::new(i.clone(), self.kademlia.lock().unwrap().get_trust_score(i.id.clone()).get_score()));
                        }
                    }
                }
            }
        }


        return Err(io::Error::new(ErrorKind::NotFound, "The block was not found"));
    }


    fn reward(&self, reroute_table: &mut HashMap<Node, Vec<Node>>, already_checked: &Vec<Node>, result_node: Node){
        let mut rewarded: Vec<Node> = Vec::new();
        let mut refs: Vec<Node> = Vec::new();
        refs.push(result_node);
        while !refs.is_empty() {
            //println!("Stuck on While. Reroute Size: {}", reroute_table.len());
            let mut new_refs: Vec<Node> = Vec::new();
            for ref_node in refs {
                if let Some(mentioning_nodes) = reroute_table.get(&ref_node) {
                    for n in mentioning_nodes {
                        if !rewarded.contains(n) {
                            new_refs.push(n.clone());
                            self.kademlia.lock().unwrap().reputation_reward(n.id.clone());
                            rewarded.push(n.clone());
                        }
                    }
                }
                reroute_table.remove(&ref_node);
            }
            refs = new_refs.clone();
            drop(new_refs);
        }

        for i in already_checked {
            if !rewarded.contains(i) {
                self.kademlia.lock().unwrap().reputation_penalty(i.id.clone());
            }
        }
    }

    fn get_batch(&self, peers: Option<&mut Vec<Node>>, map: Option<&mut BinaryHeap<NodeNewDistance>>, max: usize) -> Vec<Node> {
        // Will attempt to return up to max nodes from the provided data structure
        let mut res_list: Vec<Node> = Vec::new();
        if !peers.is_none(){
            let data_struct = peers.unwrap();
            let mut range = max;
            if data_struct.len() < max {
                range = data_struct.len();
            }
            res_list = data_struct.drain(0..range).collect();
        } else if !map.is_none(){
            let data_struct = map.unwrap();
            let mut count = 0;
            while !data_struct.is_empty() && count < max {
                res_list.push(data_struct.pop().unwrap().node);
                count += 1;
            }
        }

        return res_list;
    }
}