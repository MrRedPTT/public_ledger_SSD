use std::{io};
use std::sync::{Arc, Mutex};
use async_std::net::{TcpStream};
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{Node};
use crate::p2p::req_handler::ReqHandler;
use crate::proto::packet_sending_server::{PacketSending, PacketSendingServer};
use crate::proto::{PingPacket, PongPacket, FindNodeRequest, FindNodeResponse};


#[derive(Debug, Clone)]
pub struct Peer {
    pub node: Node,
    pub kademlia: Arc<Mutex<Kademlia>>
}

#[tonic::async_trait]
impl PacketSending for Peer {
    async fn ping(&self, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        ReqHandler::ping(self, request).await
    }

    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        ReqHandler::find_node(self, request).await
    }
}
impl Peer {

    pub async fn new(node: &Node) -> Result<Peer, io::Error> {
        Ok(Peer {
            node: node.clone(),
            kademlia: Arc::new(Mutex::new(Kademlia::new(node.clone())))
        })
    }

    pub async fn init_server(self) -> tokio::sync::oneshot::Receiver<()> {
        let node = self.node.clone();
        let server = Server::builder()
            .concurrency_limit_per_connection(40)
            .add_service(PacketSendingServer::new(self))
            .serve(format!("{}:{}", node.ip, node.port).parse().unwrap());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
            }
            let _ = shutdown_tx.send(());
        });
        shutdown_rx // Channel to receive shutdown signal from the server thread
    }
    pub async fn create_sender(&self, msg: Vec<u8>, mut stream: TcpStream) -> Result<(), io::Error>{
        todo!()
    }

    // When dealing with objects, proto struct typically require the type passed
    // to be Option<Type>
    // Therefore this function is used to wrap the object inside of Option
    fn return_option<T>(arg: T) -> Option<T> {
        Some(arg)
    }


}
