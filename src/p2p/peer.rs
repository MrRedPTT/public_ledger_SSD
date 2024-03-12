
use std::{io};
use std::sync::{Arc, Mutex};
use tokio::signal;
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::req_handler::ReqHandler;
use crate::{proto};
use crate::kademlia::auxi;
use crate::proto::packet_sending_server::{PacketSending, PacketSendingServer};
use crate::proto::{PingPacket, PongPacket, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, StoreRequest, StoreResponse};


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

    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        ReqHandler::find_value(self, request).await
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status>{
        ReqHandler::store(self, request).await
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
    pub async fn ping(&self, ip: &str, port: u32) -> Result<Response<PongPacket>, tonic::transport::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut client = proto::packet_sending_client::PacketSendingClient::connect(url).await?;

        let req = proto::PingPacket {
            src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
            dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port)
        };

        let request = tonic::Request::new(req);
        let response = client.ping(request).await.expect("Error while trying to ping");
        let input = response.get_ref();

        let res_ip = input.src.as_ref().unwrap();
        println!("Got a Pong from: {:?}", res_ip);
        Ok(response)
    }

    pub async fn find_node(&self, ip: &str, port: u32, id: Identifier) -> Result<Response<FindNodeResponse> , tonic::transport::Error>{
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut client = proto::packet_sending_client::PacketSendingClient::connect(url).await?;

        let req = proto::FindNodeRequest { // Ask for a node that the server holds
            id: id.0.to_vec(),
            src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
            dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port)
        };
        let request = tonic::Request::new(req);
        let response = client.find_node(request).await.expect("Error while trying to find_node");

        println!("Find Node Response: {:?}", response.get_ref());
        Ok(response)
    }

    pub async fn find_value(&self, ip: String, port: u32, id: Identifier) -> Result<Response<FindValueResponse> , tonic::transport::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut client = proto::packet_sending_client::PacketSendingClient::connect(url).await?;

        let req = proto::FindValueRequest {
            value_id: id.0.to_vec(),
            src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
            dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
        };

        let request = tonic::Request::new(req);
        let response = client.find_value(request).await.expect("Error while trying to find_value");

        println!("Find Node Response: {:?}", response.get_ref());
        Ok(response)

    }

    pub async fn store(&self, ip: String, port: u32, key: Identifier, value: String) -> Result<Response<StoreResponse> , tonic::transport::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut client = proto::packet_sending_client::PacketSendingClient::connect(url).await?;
        let req = StoreRequest {
            key: key.0.to_vec(),
            value,
            src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
            dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
        };

        let request = tonic::Request::new(req);
        let response = client.store(request).await.expect("Error while trying to store");

        println!("Store Response: {:?}", response.get_ref());
        Ok(response)

    }

    // When dealing with objects, proto struct typically require the type passed
    // to be Option<Type>
    // Therefore this function is used to wrap the object inside of Option
    fn return_option<T>(arg: T) -> Option<T> {
        Some(arg)
    }


}
