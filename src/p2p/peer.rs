use std::{io};
use async_std::net::{TcpStream};
use tokio::sync::oneshot;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::proto;
use crate::proto::packet_sending_server::{PacketSending, PacketSendingServer};
use crate::proto::{Address, PingPacket, PongPacket, FindNodeRequest, FindNodeResponse};


#[derive(Debug, Clone)]
pub struct Peer {
    node: Node,
    pub kademlia: Kademlia
}

#[tonic::async_trait]
impl PacketSending for Peer {
    async fn ping(&self, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        let input = request.get_ref();
        if input.src.is_none() || input.dst.is_none() {
            // Abort!
        }
        let node = self.node.clone();

        let pong = PongPacket {
            src: Self::return_option(Address{ip: node.ip, port: node.port}),
            dst: input.clone().src
        };

        println!("Got a Ping from: {}:{}", input.clone().src.unwrap().ip, input.clone().src.unwrap().port);
        println!("Sending Pong...");

        Ok(tonic::Response::new(pong))


    }

    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let input = request.get_ref();
        println!("Node: {:?} asked about node: {:?}", request.remote_addr(), input);
        let node_id = &input.id;
        let my_node = &self.node;
        let placeholder_node = Self::return_option(proto::Node { // Won't be read (used in case or error)
            id: my_node.id.0.to_vec(),
            ip: my_node.ip.clone(),
            port: self.node.port
        });

        if node_id.len() != ID_LEN {
            let response = FindNodeResponse {
                response_type: 1, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node,
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: "The supplied ID has an invalid size".to_string()
            };
            return Ok(tonic::Response::new(response));
        }
        let mut id_array: [u8; ID_LEN] = [0; ID_LEN];
        for i in 0..node_id.len() {
            id_array[i] = node_id[i];
        }

        let id = Identifier::new(id_array);
        let lookup = self.kademlia.get_node(id.clone());

        if lookup.is_none() {
            // Node not found, send k nearest nodes to
            let k_nearest = self.kademlia.get_k_nearest_to_node(id.clone());
            return if k_nearest.is_none() {
                // This means we don't have any node in our kbucket, which should not happen, however if it does
                // we define the FindNodeResponseType as UNKNOWN_TYPE which will prompt the "client" to drop the packet

                let response = FindNodeResponse {
                    response_type: 0, // UNKNOWN_TYPE => Abort this communication
                    node: placeholder_node,
                    list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}), // Won't be read
                    error: "Neither the target node or it's nearest nodes were found".to_string()
                };
                Ok(tonic::Response::new(response))
            } else {
                let mut list: Vec<proto::Node> = Vec::new();
                // The type cant be our definition of node but proto::Node
                // Therefore we need to create instances of those and place them inside a new vector
                for i in k_nearest.unwrap() {
                    list.push(proto::Node{id:i.id.0.to_vec(), ip:i.ip, port:i.port});
                }
                let response = FindNodeResponse {
                    response_type: 1, // UNKNOWN_TYPE => Abort this communication
                    node: placeholder_node,
                    list: Self::return_option(proto::KNearestNodes{nodes: list}),
                    error: "".to_string()
                };
                Ok(tonic::Response::new(response))
            }
        } else {
            // Node found, send it back
            let target_node = lookup.unwrap();
            let response = FindNodeResponse {
                response_type: 2, // Found the target node
                node: Self::return_option(proto::Node{id: target_node.id.0.to_vec(), ip: target_node.ip, port: target_node.port}),
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}), // Won't be read
                error: "".to_string()
            };
            Ok(tonic::Response::new(response))
        }

    }
}
impl Peer {

    pub async fn new(node: &Node) -> Result<Peer, io::Error> {
        Ok(Peer {
            node: node.clone(),
            kademlia: Kademlia::new(node.clone())
        })
    }

    pub async fn init_server(self) -> tokio::sync::oneshot::Receiver<()> {
        let node = self.node.clone();
        let server = Server::builder()
            .concurrency_limit_per_connection(40)
            .add_service(PacketSendingServer::new(self.clone()))
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

    // This is here since packet from proto
    // expect variable wrapped inside of Option
    fn return_option<T>(arg: T) -> Option<T> {
        Some(arg)
    }



}
