use std::{default, io};
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use log::{debug, error, info};
use tokio::signal;
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::private::req_handler::ReqHandler;
use crate::p2p::private::res_handler::ResHandler;
use crate::proto::{FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, KNearestNodes, PingPacket, PongPacket, StoreRequest, StoreResponse};
use crate::proto::packet_sending_server::{PacketSending, PacketSendingServer};

#[derive(Debug, Clone)]
pub struct Peer {
    pub node: Node,
    pub kademlia: Arc<Mutex<Kademlia>>
}

#[tonic::async_trait]
impl PacketSending for Peer {

    /// # Ping Handler
    /// This function acts like a proxy function to the [ReqHandler::ping]
    async fn ping(&self, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        ReqHandler::ping(self, request).await
    }

    /// # Store Handler
    /// This function acts like a proxy function to the [ReqHandler::store]
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        println!("DEBUG PEER::STORE_HANDLER -> Got a Store Request");
        ReqHandler::store(self, request).await
    }

    /// # Find_Node Handler
    /// This function acts like a proxy function to the [ReqHandler::find_node]
    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        ReqHandler::find_node(self, request).await
    }

    /// # Find_Value Handler
    /// This function acts like a proxy function to the [ReqHandler::find_value]
    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        ReqHandler::find_value(self, request).await
    }
}
impl Peer {

    /// # new
    /// Creates a new instance of the Peer Object
    pub async fn new(node: &Node) -> Result<Peer, io::Error> {
        Ok(Peer {
            node: node.clone(),
            kademlia: Arc::new(Mutex::new(Kademlia::new(node.clone())))
        })
    }

    /// # init_server
    /// Instantiates a tonic server that will listen for RPC calls. The information used,
    /// namely ip and port, is the one provided through the node.
    /// The server will be created on a different thread, and
    /// a different thread will also be created which only purpose is to listen
    /// for CTRL + C signals
    ///
    /// ### Returns
    /// As mentioned, this function will also create a second thread listening for signals.
    /// In order for the server to keep running, you need to block the main thread otherwise once
    /// it ends, it will shut down every other thread. For that, the second thread will return a receiver
    /// with which you can do something like `let _ = init_server().await`. This way, the main thread will
    /// be blocked, and the process will only terminate once a CTRL + C is detected.
    pub async fn init_server(self) -> tokio::sync::oneshot::Receiver<()> {
        let node = self.node.clone();
        println!("DEBUG PEER::INIT_SERVER => Creating server at {}:{}", node.ip, node.port);
        let server = Server::builder()
            .concurrency_limit_per_connection(40)
            .add_service(PacketSendingServer::new(self))
            .serve(format!("{}:{}", node.ip, node.port).parse().unwrap());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            // Wait for CTRL+C signal
            signal::ctrl_c().await.expect("failed to listen for event");
            // Send shutdown signal to the server thread
            let _ = shutdown_tx.send(());

            // Print a message indicating the server is shutting down
           info!("Shutting down server...");
        });

        tokio::spawn(async move {
            if let Err(e) = server.await {
                println!("Server error: {}", e);
            }
        });
        shutdown_rx // Channel to receive shutdown signal from the server thread
    }

    /// # Ping Request
    /// Proxy for the [ResHandler::ping] function.
    pub async fn ping(&self, ip: &str, port: u32) -> Result<Response<PongPacket>, io::Error> {
        ResHandler::ping(self, ip, port).await
    }

    #[async_recursion]
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
    pub async fn find_node(&self, id: Identifier, peers: Option<Vec<Node>>, already_checked: Option<Vec<Node>>) -> Result<Response<FindNodeResponse> , io::Error>{

        let mut nodes = peers.clone(); // This will argument is passed so that this function can be used recursively
        let mut node_list: Vec<Node> = Vec::new();
        let mut already_checked_list: Vec<Node> = Vec::new();
        if peers.is_none() {
            debug!("DEBUG PEER::FIND_NODE => No nodes as arguments passed");
            nodes = self.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone());
        } else {
            debug!("DEBUG PEER::FIND_NODE => Argument Nodes: {:?}", peers.clone().unwrap());
            if !already_checked.is_none() {
                already_checked_list = already_checked.unwrap();
            }
        }
        let mut arguments: Vec<(String, u32)> = Vec::new();
        if nodes.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "No nodes found"));
        } else {
            node_list = nodes.unwrap();
          for i in &node_list{
              debug!("DEBUGG IN PEER::FIND_NODE -> Peers discovered: {}:{}", i.ip, i.port);
              arguments.push((i.ip.clone(), i.port))
          }
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Limit the amount of threads

        // Process tasks concurrently using Tokio
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
                    res
                })
            })
            .collect::<Vec<_>>();

        // Output results
        let mut errors: Vec<Node> = Vec::new();
        let mut query_result: Option<Response<FindNodeResponse>> = None;
        let mut counter: usize = 0;
        for task in tasks {
            let result = task.await.expect("Failed to retrieve task result");
            match result {
                Err(e) => {
                    error!("Error found: {}", e);
                }
                Ok(res) => {
                    if res.get_ref().response_type == 2 {
                        debug!("DEBUG PEER::FIND_NODE -> Find the parça");
                        query_result = Some(res);
                        break;
                    } else {
                        for n in <std::option::Option<KNearestNodes> as Clone>::clone(&res.get_ref().list).unwrap().nodes {
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
            for i in &errors {
                info!("Node: {}:{}", i.ip, i.port);
            }
            // Check if the list is empty, if true, send None
            // otherwise send the already_checked_list as argument
            if already_checked_list.len() == 0 {
                Self::find_node(self, id, Some(errors), None).await
            } else {
                Self::find_node(self, id, Some(errors), Some(already_checked_list)).await
            }

        }
    }

    /// # Find Value Request
    /// Proxy for the [ResHandler::find_value] function.
    pub async fn find_value(&self, ip: String, port: u32, id: Identifier) -> Result<Response<FindValueResponse> , io::Error> {
        ResHandler::find_value(self, ip, port, id).await
    }

    /// # Store Request
    /// Proxy for the [ResHandler::store] function.
    pub async fn store(&self, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {

        let nodes = self.kademlia.lock().unwrap().get_k_nearest_to_node(key.clone());
        let mut node_list: Vec<Node> = Vec::new();
        let mut arguments: Vec<(String, u32)> = Vec::new();
        if nodes.is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "No nodes found"));
        } else {
            node_list = nodes.unwrap();
            debug!("DEBUGG IN PEER::STORE -> NodeList len: {}", node_list.len());
            println!("DEBUGG IN PEER:STORE -> Contains server3: {}", node_list.contains(&Node::new("127.0.46.1".to_string(), 8935).unwrap()));
            for i in &node_list{
                debug!("DEBUGG IN PEER::STORE -> Peers discovered: {}:{}", i.ip, i.port);
                arguments.push((i.ip.clone(), i.port))
            }
        }
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Limit the amount of threads

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
                    res
                })
            })
            .collect::<Vec<_>>();

        // Output results
        let mut errors: Vec<Node> = Vec::new();
        let mut query_result: Option<Response<StoreResponse>> = None;
        let mut counter: usize = 0;
        for task in tasks {
            let result = task.await.expect("Failed to retrieve task result");
            match result {
                Err(e) => {
                    error!("Error found: {}", e);
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



}
