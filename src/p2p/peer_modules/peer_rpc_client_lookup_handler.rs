use std::collections::HashMap;
use std::sync::Arc;

use log::debug;
use log::info;

use crate::kademlia::node::{Identifier, Node};
use crate::ledger::block::Block;
use crate::p2p::peer::Peer;
use crate::p2p::private::req_handler_modules::res_handler::ResHandler;

impl Peer {
    /// # Find Node Request
    /// The actual logic used to send the request is defined in [ResHandler::find_node]
    /// This function will look at the node provide, determine which nodes it should contact in
    /// order to obtain the information and then send the request to those nodes in parallel
    /// If any of the nodes return the target node then simply return it back, otherwise, collect the
    /// forwarding nodes returned, remove duplicates, and call this function recursively over those nodes
    /// and do that over and over again until either, the target node is found, or the nodes stop responding
    ///
    /// #### Info
    /// If you're calling this function both the [peers] and [already_checked] arguments should be passed as [None]
    pub async fn find_node_handler(&self, id: Identifier, peers: Vec<Node>, already_checked: &mut Vec<Node>, recommended_map: &mut HashMap<Node, Vec<Node>>) -> Result<Option<(Node, Node)>, Option<Vec<Node>>> {


        for peer in &peers {
            self.kademlia.lock().unwrap().increment_interactions(peer.id.clone());
            self.kademlia.lock().unwrap().increment_lookups(peer.id.clone());
            self.kademlia.lock().unwrap().send_back_specific_node(&peer);
        }

        let mut arguments: Vec<(String, u32, Node)> = Vec::new();
        for i in &peers{
            if !already_checked.contains(i) {
                arguments.push((i.ip.clone(), i.port, i.clone()));
                already_checked.push(i.clone());
            }
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(7)); // Limit the number of threads

        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let node = self.node.clone();
                let ident = id.clone();
                let own_id = self.id.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::find_node(&node, arg.0.as_ref(), arg.1, &ident, &own_id.clone()).await;
                    drop(permit);
                    (res, arg.2)
                })
            })
            .collect::<Vec<_>>();


        let mut recommendation: Vec<Node> = Vec::new();
        for task in tasks {
            let (res, node) = task.await.expect("Task failed");
            match res {
                Ok(result) => {
                    if result.get_ref().response_type == 2 && !result.get_ref().node.is_none() && result.get_ref().clone().node.unwrap().id == id.0.to_vec(){
                        info!("Found the node");
                        if let Some(target_node) = result.get_ref().clone().node {
                            let n = Node::new(target_node.ip, target_node.port).unwrap();
                            return Ok(Some((n, node.clone())));
                        }
                        return Err(None);
                    } else if result.get_ref().response_type == 1 {
                        info!("Got K nearest nodes");
                        let knearest = result.get_ref().clone().list;
                        if knearest.is_none() {
                            return Err(None);
                        }
                        let list = knearest.unwrap().nodes;
                        let temp = &mut Vec::new();
                        temp.push(node.clone());
                        for i in list {
                            let node = Node::new(i.ip, i.port).unwrap();
                            // Add the recommendation
                            if let Some(list_of_nodes) = recommended_map.get_mut(&node) {
                                temp.extend(list_of_nodes.clone());
                            } else {
                                // If Node X does not exist in the map, insert it with the new mentioning nodes
                                recommended_map.insert(node.clone(), temp.to_vec());
                            }
                            recommendation.push(node.clone())
                        }
                    } else {
                        debug!("An error has occured during the processing of the request");
                    }

                },
                Err(_) => {
                    debug!("Got an error on the response");
                    self.kademlia.lock().unwrap().risk_penalty(node.id);
                }
            }
        }

        return Err(Some(recommendation));


    }

    pub async fn find_value_handler(&self, id: Identifier, peers: Vec<Node>, already_checked: &mut Vec<Node>, recommended_map: &mut HashMap<Node, Vec<Node>>) -> Result<Option<(String, Node)>, Option<Vec<Node>>> {

        for peer in &peers {
            self.kademlia.lock().unwrap().increment_interactions(peer.id.clone());
            self.kademlia.lock().unwrap().increment_lookups(peer.id.clone());
            self.kademlia.lock().unwrap().send_back_specific_node(&peer);
            already_checked.push(peer.clone());
        }

        let mut arguments: Vec<(String, u32, Node)> = Vec::new();
        for i in &peers{
            arguments.push((i.ip.clone(), i.port, i.clone()));
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(7)); // Limit the number of threads

        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let node = self.node.clone();
                let ident = id.clone();
                let own_id = self.id.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::find_value(&node, arg.0.as_ref(), arg.1, &ident, &own_id.clone()).await;
                    drop(permit);
                    (res, arg.2)
                })
            })
            .collect::<Vec<_>>();


        let mut recommendation: Vec<Node> = Vec::new();
        for task in tasks {
            let (res, node) = task.await.expect("Task failed");
            match res {
                Ok(result) => {
                    if result.get_ref().response_type == 2 && result.get_ref().value.clone() != "".to_string(){
                        info!("Found the value");
                        let target_value = result.get_ref().clone().value;
                        return Ok(Some((target_value, node.clone())));
                    } else if result.get_ref().response_type == 1 {
                        info!("Got K nearest nodes");
                        let knearest = result.get_ref().clone().list;
                        if knearest.is_none() {
                            return Err(None);
                        }
                        let list = knearest.unwrap().nodes;
                        let temp = &mut Vec::new();
                        temp.push(node.clone());
                        for i in list {
                            let node = Node::new(i.ip, i.port).unwrap();
                            // Add the recommendation
                            if let Some(list_of_nodes) = recommended_map.get_mut(&node) {
                                temp.extend(list_of_nodes.clone());
                            } else {
                                // If Node X does not exist in the map, insert it with the new mentioning nodes
                                recommended_map.insert(node.clone(), temp.to_vec());
                            }
                            recommendation.push(node.clone())
                        }
                    } else {
                        debug!("An error has occured during the processing of the request");
                    }

                },
                Err(_) => {
                    debug!("Got an error on the response");
                    self.kademlia.lock().unwrap().risk_penalty(node.id);
                }
            }
        }

        return Err(Some(recommendation));


    }
    pub async fn get_block_handler(&self, id: String, peers: Vec<Node>, already_checked: &mut Vec<Node>, recommended_map: &mut HashMap<Node, Vec<Node>>) -> Result<Option<(Block, Node)>, Option<Vec<Node>>> {
        for peer in &peers {
            self.kademlia.lock().unwrap().increment_interactions(peer.id.clone());
            self.kademlia.lock().unwrap().increment_lookups(peer.id.clone());
            self.kademlia.lock().unwrap().send_back_specific_node(&peer);
            already_checked.push(peer.clone());
        }

        let mut arguments: Vec<(String, u32, Node)> = Vec::new();
        for i in &peers{
            arguments.push((i.ip.clone(), i.port, i.clone()));
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(7)); // Limit the number of threads

        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let node = self.node.clone();
                let ident = id.clone();
                let own_id = self.id.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::get_block(&node, arg.0.as_ref(), arg.1, &ident, &own_id.clone()).await;
                    drop(permit);
                    (res, arg.2)
                })
            })
            .collect::<Vec<_>>();


        let mut recommendation: Vec<Node> = Vec::new();
        for task in tasks {
            let (res, node) = task.await.expect("Task failed");
            match res {
                Ok(result) => {
                    if result.get_ref().response_type == 2 && !result.get_ref().block.is_none(){
                        info!("Found the Block");
                        if let Some(target_block) = result.get_ref().clone().block {
                            return Ok(Some((Block::proto_to_block(target_block), node.clone())));
                        }
                        return Err(None);
                    } else if result.get_ref().response_type == 1 {
                        info!("Got K nearest nodes");
                        let knearest = result.get_ref().clone().list;
                        if knearest.is_none() {
                            return Err(None);
                        }
                        let list = knearest.unwrap().nodes;
                        let temp = &mut Vec::new();
                        temp.push(node.clone());
                        for i in list {
                            let node = Node::new(i.ip, i.port).unwrap();
                            // Add the recommendation
                            if let Some(list_of_nodes) = recommended_map.get_mut(&node) {
                                temp.extend(list_of_nodes.clone());
                            } else {
                                // If Node X does not exist in the map, insert it with the new mentioning nodes
                                recommended_map.insert(node.clone(), temp.to_vec());
                            }
                            recommendation.push(node.clone())
                        }
                    } else {
                        debug!("An error has occurred during the processing of the request");
                    }

                },
                Err(_) => {
                    debug!("Got an error on the response");
                    self.kademlia.lock().unwrap().risk_penalty(node.id);
                }
            }
        }

        return Err(Some(recommendation));


    }
}