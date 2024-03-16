
use std::{io};
use std::sync::{Arc, Mutex};
use tokio::signal;
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{Identifier, Node};
use crate::proto;
use crate::p2p::private::req_handler::ReqHandler;
use crate::p2p::private::res_handler::ResHandler;
use crate::proto::packet_sending_server::{PacketSending, PacketSendingServer};
use crate::proto::{PingPacket, PongPacket, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, StoreRequest, StoreResponse};

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

    /// # Store Handler
    /// This function acts like a proxy function to the [ReqHandler::store]
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status>{
        ReqHandler::store(self, request).await
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
            println!("Shutting down server...");
        });

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
            }
        });
        shutdown_rx // Channel to receive shutdown signal from the server thread
    }

    /// # Ping Request
    /// Proxy for the [ResHandler::ping] function.
    pub async fn ping(&self, ip: &str, port: u32) -> Result<Response<PongPacket>, io::Error> {
        ResHandler::ping(self, ip, port).await
    }

    /// # Find Node Request
    /// Proxy for the [ResHandler::find_node] function.
    pub async fn find_node(&self, ip: &str, port: u32, id: Identifier) -> Result<Response<FindNodeResponse> , io::Error>{
        ResHandler::find_node(self, ip, port, id).await
    }

    /// # Find Value Request
    /// Proxy for the [ResHandler::find_value] function.
    pub async fn find_value(&self, ip: String, port: u32, id: Identifier) -> Result<Response<FindValueResponse> , io::Error> {
        ResHandler::find_value(self, ip, port, id).await
    }

    /// # Store Request
    /// Proxy for the [ResHandler::store] function.
    pub async fn store(&self, ip: String, port: u32, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {
        ResHandler::store(self, ip, port, key, value).await
    }



}
