use std::env;
use std::sync::Arc;

use log::{debug, info};
use tonic::Request;
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

use crate::{auxi, proto};
use crate::kademlia::node::{ID_LEN, Node};
use crate::ledger::block::Block;
use crate::marco::marco::Marco;
use crate::p2p::peer::{Peer, TTL};
use crate::proto::{BlockBroadcast, MarcoBroadcast, SrcAddress};

pub struct BroadCastReq {}

impl BroadCastReq {
    pub async fn broadcast(peer: &Peer, transaction: Option<Marco>, block: Option<Block>, ttl: Option<u32>, block_request: Option<Request<BlockBroadcast>>, trans_request: Option<Request<MarcoBroadcast>>) {
        let mut time_to_live: u32 = TTL;
        if !ttl.is_none() {
            time_to_live = ttl.unwrap();
        }


        let all_nodes = peer.kademlia.lock().unwrap().get_all_nodes();
        if all_nodes.is_none() {
            // No nodes to propagate to
            return;
        }
        let mut nodes = all_nodes.unwrap();

        // Remove the sender and own node from the list of targets
        let mut sender;
        let cert;
        if block_request.is_none() && trans_request.is_none() {
            sender = None;
            let mut slash = "/";
            if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "windows" {
                slash = "\\";
            }
            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to read ca.pem");
            cert = pem
        }
        else if trans_request.is_some(){
            sender = trans_request.as_ref().unwrap().get_ref().clone().src;
            cert = trans_request.as_ref().unwrap().get_ref().clone().cert;
            if format!("{}:{}", sender.clone().unwrap().ip, sender.clone().unwrap().port) == format!("{}:{}", peer.node.ip.clone(), peer.node.port.clone()) {
                // Means we received the request with source ourselves
                return;
            }
        } else {
            sender = block_request.as_ref().unwrap().get_ref().clone().src;
            cert = block_request.as_ref().unwrap().get_ref().clone().cert;
            if format!("{}:{}", sender.clone().unwrap().ip, sender.clone().unwrap().port) == format!("{}:{}", peer.node.ip.clone(), peer.node.port.clone()) {
                // Means we received the request with source ourselves
                return;
            }
        }
        if !sender.is_none(){
            let sender_node = &sender.clone().unwrap().clone();
            nodes.retain(|x| (x.ip != sender_node.ip.to_string() && x.port != sender_node.port) && (String::from(x.ip.clone()) != String::from(&peer.node.ip) || x.port != peer.node.port));
        } else {
            nodes.retain(|x| String::from(x.ip.clone()) != String::from(&peer.node.ip) || x.port != peer.node.port);
            sender = auxi::gen_address_src(peer.id.clone(), peer.node.ip.clone(), peer.node.port.clone());
        }

        if nodes.len() == 0 {
            // Call bootstrap to refresh nodes
            peer.boot().await;
            return;
        }
        let n = ( (ID_LEN / 16) as f32).ceil() as usize ; // Number of sub-vectors

        let mut chunk_size = (nodes.len() / n) + 1;
        if chunk_size <= 0 {
            chunk_size = 1;
        }
        let mut chunks = nodes.chunks(chunk_size);

        // This will divide the nodes into chunks
        // This way we can divide them into threads
        let mut sub_vectors = Vec::new();
        for _ in 0..n {
            if let Some(chunk) = chunks.next() {
                sub_vectors.push(chunk.to_vec());
            }
        }


        let mut arguments: Vec<Vec<Node>> = Vec::new();
        if sub_vectors.len() > 0 {
            for i in &sub_vectors{
                arguments.push(i.clone())
            }
        }


        let semaphore = Arc::new(tokio::sync::Semaphore::new(16)); // Limit the number of threads

        // Process tasks concurrently using Tokio
        let tasks = arguments.into_iter()
            .map(|arg| {
                let semaphore = semaphore.clone();
                let time = time_to_live.clone();
                let trans = transaction.clone();
                let bl = block.clone();
                let send = sender.clone();
                let certi = cert.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    for target in arg {
                        info!("Sending broadcast to {}:{}", target.ip.clone(), target.port.clone());
                        if trans.is_none(){
                            Self::send_request(target.ip, target.port, None, bl.clone(), time, send.clone().unwrap(), certi.clone()).await;
                        } else {
                            Self::send_request(target.ip, target.port, trans.clone(), None, time, send.clone().unwrap(), certi.clone()).await;
                        }
                    }
                    drop(permit);
                })
            })
            .collect::<Vec<_>>();

        for task in tasks {
            let _ = task.await.expect("Failed to retrieve task return");
        }

        // Propagate message
        if transaction.is_none(){
            info!("Block propagated!");
        } else {
            info!("Transaction propagated!");
        }

    }

    async fn send_request(ip: String, port: u32, marco_op: Option<Marco>, block_op: Option<Block>, ttl: u32, sender: SrcAddress, cert: String) {
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
            let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to open server.crt");
            let client_key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key"))).expect("Failed to open server.key");
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
                Err(_) => {
                    debug!("Error while broadcasting for {url}");
                    return;
                }
                Ok(channel) => { channel }
            };

            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            if marco_op.is_none() {
                let block = block_op.unwrap();
                let mut trans: Vec<proto::Marco> = Vec::new();
                for i in block.transactions {
                    let data = auxi::transform_marco_to_proto(&i);
                    trans.push(data);
                }
                let req = proto::BlockBroadcast {
                    src: Some(sender.clone()),
                    dst: auxi::gen_address_dst(ip.to_string(), port),
                    block: Some(proto::Block {
                        hash: block.hash,
                        index: block.index as u64,
                        timestamp: block.timestamp,
                        prev_hash: block.prev_hash,
                        nonce: block.nonce,
                        difficulty: block.difficulty as u64,
                        miner_id: block.miner_id,
                        merkle_tree_root: block.merkle_tree_root,
                        confirmations: 0,
                        transactions: trans,
                    }),
                    ttl,
                    cert
                };
                let request = tonic::Request::new(req);
                let res = c.send_block(request).await;
                match res {
                    Err(e) => {
                        debug!("An error has occurred while trying to broadcast Block: {{{}}}", e);
                        return;
                    },
                    Ok(_) => {
                        info!("Send Broadcast Returned");
                        return;
                    }
                }
            } else {
                let transaction = marco_op.unwrap();
                let req = proto::MarcoBroadcast { // Ask for a node that the server holds
                    src: Some(sender),
                    dst: auxi::gen_address_dst(ip.to_string(), port),
                    cert,
                    marco: Some(auxi::transform_marco_to_proto(&transaction)),
                    ttl
                };
                let request = tonic::Request::new(req);
                let res = c.send_marco(request).await;
                match res {
                    Err(e) => {
                        debug!("An error has occurred while trying to broadcast Transaction: {{{}}}", e);
                        return;
                    },
                    Ok(_) => {
                        info!("Send Broadcast Returned");
                        return;
                    }
                }
            }
        } else {
            // Un-Encrypted communication is no longer supported
            return;
        }
    }

}
