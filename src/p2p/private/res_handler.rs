use std::io;
use std::io::{Error, ErrorKind};

use egui::mutex::MutexGuard;
use log::{debug, error, info};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tonic::Response;

use crate::auxi;
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;
use crate::proto;
use crate::proto::{FindNodeResponse, FindValueResponse, PongPacket, StoreRequest, StoreResponse};

pub(crate) struct ResHandler{}

impl  ResHandler {

    /// # Ping
    /// Sends a ping request to http://ip:port using the gRPC procedures
    ///
    /// ### Returns
    /// This will either return a [proto::PongPacket], indicating a valid response from the target, or an Error which can be caused
    /// by either problems in the connections or a Status returned by the receiver. In either case, the error message is returned.
    pub(crate) async fn ping(peer: &Peer, ip: &str, port: u32) -> Result<Response<PongPacket>, io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        // Generate a random string
        let rand_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let randID = auxi::gen_id(rand_str);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;

        match c {
            Err(e) => {
                error!("An error has occurred while trying to establish a connection for find node: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::PingPacket {
                    src: auxi::gen_address(peer.node.id.clone(), peer.node.ip.clone(), peer.node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                    rand_id: randID.0.to_vec()
                };

                let request = tonic::Request::new(req);
                let res = client.ping(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to ping: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        info!("Ping Response: {:?}", response.get_ref());
                        if response.get_ref().rand_id != randID.0.to_vec() {
                            return Err(io::Error::new(ErrorKind::InvalidData, "The random ID provided on the Ping was not echoed back"));
                        }
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

    pub async fn find_node(node: &Node, ip: &str, port: u32, id: &Identifier) -> Result<Response<FindNodeResponse>, Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);
        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;
        match c {
            Err(e) => {
                error!("An error has occurred while trying to establish a connection for find node: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::FindNodeRequest { // Ask for a node that the server holds
                    id: id.0.to_vec(),
                    src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port)
                };
                let request = tonic::Request::new(req);
                let res = client.find_node(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to find node: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        info!("Find node Response: {:?}", response.get_ref());
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
    pub(crate) async fn find_value(node: &Node, ip: &str, port: u32, id: &Identifier) -> Result<Response<FindValueResponse> , io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;

        match c {
            Err(e) => {
                error!("An error has occurred while trying to establish a connection for find value: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = proto::FindValueRequest {
                    value_id: id.0.to_vec(),
                    src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                };

                let request = tonic::Request::new(req);
                let res = client.find_value(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to find value: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        info!("Find Value Response: {:?}", response.get_ref());
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
    pub(crate) async fn store(node: &Node, ip: String, port: u32, key: Identifier, value: String) -> Result<Response<StoreResponse> , io::Error> {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);

        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;
        match c {
            Err(e) => {
                error!("An error has occurred while trying to establish a connection for store: {}", e);
                Err(io::Error::new(ErrorKind::ConnectionRefused, e))
            },
            Ok(mut client) => {
                let req = StoreRequest {
                    key: key.0.to_vec(),
                    value,
                    src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                };

                let request = tonic::Request::new(req);
                let res = client.store(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to store value: {{{}}}", e);
                        Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                    },
                    Ok(response) => {
                        info!("Store Response: {:?}", response.get_ref());
                        Ok(response)
                    }
                }
            }
        }


    }

}