
use std::{io};
use std::io::ErrorKind;
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

    /// # Ping
    /// Sends a ping request to http://ip:port using the gRPC procedures
    ///
    /// ### Returns
    /// This will either return a [proto::PongPacket], indicating a valid response from the target, or an Error which can be caused
    /// by either problems in the connections or a Status returned by the receiver. In either case, the error message is returned.
    pub async fn ping(&self, ip: &str, port: u32) -> Result<Response<PongPacket>, io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;

        match c {
            Err(e) => {
                eprintln!("An error has occurred while trying to establish a connection for find node: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::PingPacket {
                    src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port)
                };

                let request = tonic::Request::new(req);
                let res = client.ping(request).await;
                match res {
                    Err(e) => {
                        eprintln!("An error has occurred while trying to ping: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        println!("Ping Response: {:?}", response.get_ref());
                        Ok(response)
                    }
                }
            }
        }
    }

    /// # find_node
    /// Sends the find_node gRPC procedure to the target (http://ip:port). The goal of this procedure is to find information about a given node
    /// `id`. If the receiver holds the node queried (by the id) returns the node object, otherwise returns the k nearest nodes to the target id.
    ///
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::FindNodeResponse].
    pub async fn find_node(&self, ip: &str, port: u32, id: Identifier) -> Result<Response<FindNodeResponse> , io::Error>{
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;

        match c {
            Err(e) => {
                eprintln!("An error has occurred while trying to establish a connection for find node: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::FindNodeRequest { // Ask for a node that the server holds
                    id: id.0.to_vec(),
                    src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port)
                };

                let request = tonic::Request::new(req);
                let res = client.find_node(request).await;
                match res {
                    Err(e) => {
                        eprintln!("An error has occurred while trying to find node: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        println!("Find node Response: {:?}", response.get_ref());
                        Ok(response)
                    }
                }
            }
        }
    }

    /// # find_value
    /// Similar to find_node, this gRPC procedure will query a node for value with key [Identifier] and if the receiver node holds that
    /// entry, returns it, otherwise returns the k nearest nodes to the key.
    ///
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::FindValueResponse].
    pub async fn find_value(&self, ip: String, port: u32, id: Identifier) -> Result<Response<FindValueResponse> , io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;

        match c {
            Err(e) => {
                eprintln!("An error has occurred while trying to establish a connection for find value: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::FindValueRequest {
                    value_id: id.0.to_vec(),
                    src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                };

                let request = tonic::Request::new(req);
                let res = client.find_value(request).await;
                match res {
                    Err(e) => {
                        eprintln!("An error has occurred while trying to find value: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        println!("Find Value Response: {:?}", response.get_ref());
                        Ok(response)
                    }
                }
            }
        }

    }

    /// # store
    /// This gRPC procedure is only non-query one and so has a slightly different behavior where we request a node to store a certain (key ([Identifier]), Value ([String]))
    /// and on the receiver side, if the receiver is the closest node to the key than stores it, otherwise the receiver itself will forward the key to the k nearest nodes.
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::StoreResponse].
    pub async fn store(&self, ip: String, port: u32, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;
        match c {
            Err(e) => {
                eprintln!("An error has occurred while trying to establish a connection for store: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = StoreRequest {
                    key: key.0.to_vec(),
                    value,
                    src: auxi::gen_address(self.node.id.clone(), self.node.ip.clone(), self.node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                };

                let request = tonic::Request::new(req);
                let res = client.store(request).await;
                match res {
                    Err(e) => {
                        eprintln!("An error has occurred while trying to store value: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        println!("Store Response: {:?}", response.get_ref());
                        Ok(response)
                    }
                }
            }
        }


    }



}
