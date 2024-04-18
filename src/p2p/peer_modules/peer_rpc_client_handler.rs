use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use async_recursion::async_recursion;
use log::{debug, error, info};
use tonic::Response;

use crate::kademlia::node;
use crate::kademlia::node::{Identifier, Node};
use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::Peer;
use crate::p2p::private::broadcast_api::BroadCastReq;
use crate::p2p::private::res_handler::ResHandler;
use crate::proto::{FindNodeResponse, FindValueResponse, KNearestNodes, PongPacket, StoreResponse};

impl Peer {
    /// # Ping Request
    /// Proxy for the [ResHandler::ping] function.
    pub async fn ping(&self, ip: &str, port: u32, id: Identifier) -> Result<Response<PongPacket>, io::Error> {
        self.kademlia.lock().unwrap().increment_interactions(id.clone());
        let res = ResHandler::ping(self, ip, port).await;
        match res {
            Err(e) => {
                self.kademlia.lock().unwrap().risk_penalty(id.clone());
                Err(e)
            },
            Ok(res) => return Ok(res)
        }
    }


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
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::find_node(&node, arg.0.as_ref(), arg.1, &ident).await;
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
                        let mut temp = &mut Vec::new();
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
                        error!("An error has occured during the processing of the request");
                    }

                },
                Err(e) => {
                    error!("Got an error on the response");
                    self.kademlia.lock().unwrap().risk_penalty(node.id);
                }
            }
        }

        return Err(Some(recommendation));


    }

    #[async_recursion]
    /// # Find Value Request
    /// Proxy for the [ResHandler::find_value] function.
    pub async fn find_value(&self, id: Identifier, peers: Option<Vec<Node>>, already_checked: Option<Vec<Node>>) -> Result<Response<FindValueResponse> , io::Error> {
        let mut nodes = peers.clone(); // This will argument is passed so that this function can be used recursively
        let mut node_list: Vec<Node> = Vec::new();
        let mut already_checked_list: Vec<Node> = Vec::new();
        if peers.is_none() {
            debug!("DEBUG PEER::FIND_VALUE => No nodes as arguments passed");
            nodes = self.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone());
        } else {
            debug!("DEBUG PEER::FIND_VALUE => Argument Nodes: {:?}", peers.clone().unwrap());
            if !already_checked.is_none() {
                already_checked_list = already_checked.unwrap();
            }
        }
        let mut arguments: Vec<(String, u32, Identifier)> = Vec::new();
        if nodes.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "No nodes found"));
        } else {
            node_list = nodes.unwrap();
            for i in &node_list{
                debug!("DEBUGG IN PEER::FIND_VALUE -> Peers discovered: {}:{}", i.ip, i.port);
                arguments.push((i.ip.clone(), i.port, i.id.clone()));
                self.kademlia.lock().unwrap().increment_interactions(i.clone().id);
            }
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Limit the number of threads

        // Process tasks concurrently using Tokio
        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let node = self.node.clone();
                let ident = id.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::find_value(&node, arg.0.as_ref(), arg.1, &ident).await;
                    drop(permit);
                    (res, arg.2)
                })
            })
            .collect::<Vec<_>>();

        // Output results
        let mut errors: Vec<Node> = Vec::new();
        let mut query_result: Option<Response<FindValueResponse>> = None;
        let mut counter: usize = 0;
        for task in tasks {
            let (result, id) = task.await.expect("Failed to retrieve task result");
            match result {
                Err(e) => {
                    error!("Error found: {}", e);
                    self.kademlia.lock().unwrap().risk_penalty(id);
                }
                Ok(res) => {
                    if res.get_ref().response_type == 2 {
                        debug!("DEBUG PEER::FIND_VALUE -> Found the Value");
                        query_result = Some(res);
                        break;
                    } else {
                        for n in <Option<KNearestNodes> as Clone>::clone(&res.get_ref().list).unwrap().nodes {
                            let temp = Node::new(n.ip, n.port).unwrap();
                            if !errors.contains(&temp) && !already_checked_list.contains(&temp){
                                errors.push(temp.clone());
                                already_checked_list.push(temp);
                            }
                        }
                    }
                }
            }
            // Move the node we just contacted to the back of the list
            let _ = self.kademlia.lock().unwrap().send_back_specific_node(&node_list[counter]);
            counter += 1;
        }
        if !query_result.is_none(){
            return Ok(query_result.unwrap());
        } else if errors.len() == 0usize {
            return Err(io::Error::new(ErrorKind::NotFound, "Node not found"));
        } else {
            // Sort by the new distance as it's explained in the S/Kademlia paper
            let new_errors = self.kademlia.lock().unwrap().sort_by_new_distance(errors);
            if already_checked_list.len() == 0 {
                Self::find_value(self, id, Some(new_errors), None).await
            } else {
                Self::find_value(self, id, Some(new_errors), Some(already_checked_list)).await
            }

        }
    }

    /// # Store Request
    /// Proxy for the [ResHandler::store] function.
    pub async fn store(&self, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {

        let nodes = self.kademlia.lock().unwrap().get_k_nearest_to_node(key.clone());
        let mut node_list: Vec<Node> = Vec::new();
        let mut arguments: Vec<(String, u32, Identifier)> = Vec::new();
        if nodes.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "No nodes found"));
        } else {
            node_list = nodes.unwrap();
            debug!("DEBUGG IN PEER::STORE -> NodeList len: {}", node_list.len());
            debug!("DEBUGG IN PEER:STORE -> Contains server3: {}", node_list.contains(&Node::new("127.0.46.1".to_string(), 8935).unwrap()));
            for i in &node_list{
                debug!("DEBUGG IN PEER::STORE -> Peers discovered: {}:{}", i.ip, i.port);
                // Exclude the possibility of creating a loop within it self
                if i == &self.node {
                    continue;
                }
                arguments.push((i.ip.clone(), i.port, i.id.clone()));
                self.kademlia.lock().unwrap().increment_interactions(i.clone().id);
            }
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Limit the number of threads

        // Process tasks concurrently using Tokio
        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let node = self.node.clone();
                let ident = key.clone();
                let val = value.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    let res = ResHandler::store(&node, arg.0, arg.1, ident, val).await;
                    drop(permit);
                    (res, arg.2)
                })
            })
            .collect::<Vec<_>>();

        // Output results
        let mut counter: usize = 0;
        for task in tasks {
            let (result, id) = task.await.expect("Failed to retrieve task result");
            match result {
                Err(e) => {
                    error!("Error found: {}", e);
                    self.kademlia.lock().unwrap().risk_penalty(id);
                }
                Ok(res) => {
                    if res.get_ref().response_type != 0 {
                        debug!("DEBUG PEER::STORE -> Either Forwarded or Stored");
                        return Ok(res);
                    }
                }
            }
            // Move the node we just contacted to the back of the list
            let _ = self.kademlia.lock().unwrap().send_back_specific_node(&node_list[counter]);
            counter += 1;
        }

        return Err(io::Error::new(ErrorKind::NotFound, "Node not found"));
    }

    // ===================== block_chain Network APIs (Client Side) ============================ //

    pub async fn send_transaction(&self, transaction: Transaction, ttl: Option<u32>, sender: Option<Node>) {
        BroadCastReq::broadcast(self, Some(transaction), None, ttl, sender).await;
    }

    pub async fn send_block(&self, block: Block, ttl: Option<u32>, sender: Option<Node>) {
        BroadCastReq::broadcast(self, None, Some(block), ttl, sender).await;
    }
}