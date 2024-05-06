use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use log::error;
use tonic::Response;

use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;
use crate::p2p::private::req_handler_modules::res_handler::ResHandler;
use crate::proto::{PongPacket, StoreResponse};

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

    /// # Store Request
    /// Proxy for the [ResHandler::store] function.
    pub async fn store(&self, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {

        let nodes = self.kademlia.lock().unwrap().get_k_nearest_to_node(key.clone());
        let node_list: Vec<Node>;
        let mut arguments: Vec<(String, u32, Identifier)> = Vec::new();
        if nodes.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "No nodes found"));
        } else {
            node_list = nodes.unwrap();
            for i in &node_list{
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
                    let res = ResHandler::store(&node, arg.0, arg.1, ident, val, 15).await;
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

}