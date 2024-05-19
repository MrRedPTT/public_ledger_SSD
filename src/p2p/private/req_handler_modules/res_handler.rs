use std::{env, io};
use std::io::{Error, ErrorKind};

use log::{error, info};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tonic::Response;
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

use crate::auxi;
use crate::kademlia::node::{Identifier, Node};
use crate::p2p::peer::Peer;
use crate::proto;
use crate::proto::{FindNodeResponse, FindValueResponse, GetBlockResponse, PongPacket, StoreRequest, StoreResponse};

pub(crate) struct ResHandler{}

impl  ResHandler {
    /// # Ping
    /// Sends a ping request to http://ip:port using the gRPC procedures
    ///
    /// ### Returns
    /// This will either return a [proto::PongPacket], indicating a valid response from the target, or an Error which can be caused
    /// by either problems in the connections or a Status returned by the receiver. In either case, the error message is returned.
    pub(crate) async fn ping(peer: &Peer, ip: &str, port: u32) -> Result<Response<PongPacket>, io::Error> {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            // Generate a random string
            let rand_str: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            let rand_id = auxi::gen_id(rand_str);
            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let mut slash = "\\";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
                slash = "/";
            }
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}ca.crt"))).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt")))?;
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key")))?;
            let client_identity = Identity::from_pem(client_cert, client_key);

            let tls = ClientTlsConfig::new()
                .domain_name("example.com")
                .ca_certificate(ca)
                .identity(client_identity);

            let ch = tonic::transport::Channel::from_shared(url.clone())
                .unwrap()
                .tls_config(tls)
                .expect("Failed to configure TLS channel on client")
                .connect()
                .await;
            let channel = match ch {
                Err(e) => {
                    println!("Error while creating ping channel for {url}: {e}");
                    return Err(io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))
                }
                Ok(channel) => {channel}
            };
            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            let req = proto::PingPacket {
                src: auxi::gen_address(peer.node.id.clone(), peer.node.ip.clone(), peer.node.port),
                dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                rand_id: rand_id.0.to_vec()
            };
            let res = c.ping(req).await;
            match res {
                Err(e) => {
                    println!("An error has occurred while trying to ping: {{{}}}", e);
                    Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                },
                Ok(response) => {
                    info!("Ping Response: {:?}", response.get_ref());
                    if response.get_ref().rand_id != rand_id.0.to_vec() {
                        return Err(io::Error::new(ErrorKind::InvalidData, "The random ID provided on the Ping was not echoed back"));
                    }
                    Ok(response)
                }
            }
        } else {
            // Un-Encrypted communication is no longer supported
            return Err(io::Error::new(ErrorKind::ConnectionAborted, "Un-encrypted communication no longer supported, aborting ....".to_string()))
        }
    }

    /// # find_node
    /// Sends the find_node gRPC procedure to the target (http://ip:port). The goal of this procedure is to find information about a given node
    /// `id`. If the receiver holds the node queried (by the id) returns the node object, otherwise returns the k nearest nodes to the target id.
    ///
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::FindNodeResponse].

    pub async fn find_node(node: &Node, ip: &str, port: u32, id: &Identifier) -> Result<Response<FindNodeResponse>, Error> {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let mut slash = "\\";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
                slash = "/";
            }
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}ca.crt"))).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt")))?;
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key")))?;
            let client_identity = Identity::from_pem(client_cert, client_key);

            let tls = ClientTlsConfig::new()
                .domain_name("example.com")
                .ca_certificate(ca)
                .identity(client_identity);

            let ch = tonic::transport::Channel::from_shared(url.clone())
                .unwrap()
                .tls_config(tls)
                .expect("Failed to configure TLS channel on client")
                .connect()
                .await;
            let channel = match ch {
                Err(e) => {
                    error!("Error while creating find node channel for {url}");
                    return Err(io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))
                }
                Ok(channel) => {channel}
            };
            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            let req = proto::FindNodeRequest {
                id: id.0.to_vec(),
                src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
            };

            let request = tonic::Request::new(req);
            let res = c.find_node(request).await;
            match res {
                Err(e) => {
                    error!("An error has occurred while trying to find node: {{{}}}", e);
                    Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                },
                Ok(response) => {
                    info!("Find Node Response: {:?}", response.get_ref());
                    Ok(response)
                }
            }
        } else {
            // Un-Encrypted communication is no longer supported
            return Err(io::Error::new(ErrorKind::ConnectionAborted, "Un-encrypted communication no longer supported, aborting ....".to_string()))
        }
    }

    /// # find_value
    /// Similar to find_node, this gRPC procedure will query a node for value with key [Identifier] and if the receiver node holds that
    /// entry, returns it, otherwise returns the k nearest nodes to the key.
    ///
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::FindValueResponse].
    pub(crate) async fn find_value(node: &Node, ip: &str, port: u32, id: &Identifier) -> Result<Response<FindValueResponse>, io::Error> {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let mut slash = "\\";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
                slash = "/";
            }
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}ca.crt"))).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt")))?;
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key")))?;
            let client_identity = Identity::from_pem(client_cert, client_key);

            let tls = ClientTlsConfig::new()
                .domain_name("example.com")
                .ca_certificate(ca)
                .identity(client_identity);

            let ch = tonic::transport::Channel::from_shared(url.clone())
                .unwrap()
                .tls_config(tls)
                .expect("Failed to configure TLS channel on client")
                .connect()
                .await;
            let channel = match ch {
                Err(e) => {
                    error!("Error while creating find value channel for {url}");
                    return Err(io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))
                }
                Ok(channel) => {channel}
            };
            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            let req = proto::FindValueRequest {
                value_id: id.0.to_vec(),
                src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
            };

            let request = tonic::Request::new(req);
            let res = c.find_value(request).await;
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
        } else {
            // Un-Encrypted communication is no longer supported
            return Err(io::Error::new(ErrorKind::ConnectionAborted, "Un-encrypted communication no longer supported, aborting ....".to_string()))
        }
    }

    /// # store
    /// This gRPC procedure is only non-query one and so has a slightly different behavior where we request a node to store a certain (key ([Identifier]), Value ([String]))
    /// and on the receiver side, if the receiver is the closest node to the key than stores it, otherwise the receiver itself will forward the key to the k nearest nodes.
    /// ### Returns
    /// This function can either return an error, from connection or packet-related issues, or a [proto::StoreResponse].
    pub(crate) async fn store(node: &Node, ip: String, port: u32, key_id: Identifier, value: String, ttl: u32) -> Result<Response<StoreResponse>, io::Error> {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let mut slash = "\\";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
                slash = "/";
            }
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}ca.crt"))).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt")))?;
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key")))?;
            let client_identity = Identity::from_pem(client_cert, client_key);

            let tls = ClientTlsConfig::new()
                .domain_name("example.com")
                .ca_certificate(ca)
                .identity(client_identity);

            let ch = tonic::transport::Channel::from_shared(url.clone())
                .unwrap()
                .tls_config(tls)
                .expect("Failed to configure TLS channel on client")
                .connect()
                .await;
            let channel = match ch {
                Err(e) => {
                    error!("Error while creating store channel for {url}");
                    return Err(io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))
                }
                Ok(channel) => {channel}
            };
            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            let req = StoreRequest {
                key: key_id.0.to_vec(),
                value,
                src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                ttl
            };

            let request = tonic::Request::new(req);
            let res = c.store(request).await;
            match res {
                Err(e) => {
                    error!("An error has occurred while trying to store value: {{{}}}", e);
                    Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                },
                Ok(response) => {
                    Ok(response)
                }
            }
        } else {
            // Un-Encrypted communication is no longer supported
            return Err(io::Error::new(ErrorKind::ConnectionAborted, "Un-encrypted communication no longer supported, aborting ....".to_string()))
        }
    }

    pub(crate) async fn get_block(node: &Node, ip: &str, port: u32, id: &String) -> Result<Response<GetBlockResponse>, io::Error> {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let mut slash = "\\";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
                slash = "/";
            }
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}ca.crt"))).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt")))?;
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key")))?;
            let client_identity = Identity::from_pem(client_cert, client_key);

            let tls = ClientTlsConfig::new()
                .domain_name("example.com")
                .ca_certificate(ca)
                .identity(client_identity);

            let ch = tonic::transport::Channel::from_shared(url.clone())
                .unwrap()
                .tls_config(tls)
                .expect("Failed to configure TLS channel on client")
                .connect()
                .await;
            let channel = match ch {
                Err(e) => {
                    error!("Error while creating get block channel for {url}");
                    return Err(io::Error::new(ErrorKind::ConnectionRefused, e.to_string()))
                }
                Ok(channel) => {channel}
            };

            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            let req = proto::GetBlockRequest {
                src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                id: id.clone(),
            };

            let request = tonic::Request::new(req);
            let res = c.get_block(request).await;
            match res {
                Err(e) => {
                    error!("An error has occurred while trying to get block: {{{}}}", e);
                    Err(io::Error::new(ErrorKind::ConnectionAborted, e))
                },
                Ok(response) => {
                    info!("Get Block Response: {:?}", response.get_ref());
                    Ok(response)
                }
            }
        } else {
            // Un-Encrypted communication is no longer supported
            return Err(io::Error::new(ErrorKind::ConnectionAborted, "Un-encrypted communication no longer supported, aborting ....".to_string()))
        }
    }
}