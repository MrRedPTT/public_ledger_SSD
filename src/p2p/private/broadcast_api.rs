use std::sync::Arc;

use log::{error, info};
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

use crate::{auxi, proto};
use crate::kademlia::node::{ID_LEN, Node};
use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::{Peer, TTL};

pub struct BroadCastReq {}

impl BroadCastReq {
    pub async fn broadcast(peer: &Peer, transaction: Option<Transaction>, block: Option<Block>, ttl: Option<u32>, sender: Option<Node>) {
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
        if !sender.is_none(){
            let sender_node = &sender.unwrap();
            nodes.retain(|x| x != sender_node && x != &peer.node);
        } else {
            nodes.retain(|x| x != &peer.node);
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
                let node = peer.node.clone();
                let time = time_to_live.clone();
                let trans = transaction.clone();
                let bl = block.clone();
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    for target in arg {
                        if trans.is_none(){
                            Self::send_request(&node, target.ip, target.port, None, bl.clone(), time).await;
                        } else {
                            Self::send_request(&node, target.ip, target.port, trans.clone(), None, time).await;
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
            println!("Block propagated!");
        } else {
            println!("Transaction propagated!");
        }

    }

    async fn send_request(node: &Node, ip: String, port: u32, transaction_op: Option<Transaction>, block_op: Option<Block>, ttl: u32) {
        if std::env!("TLS").to_string() == "1" {
            let mut url = "https://".to_string();
            url += &format!("{}:{}", ip, port);

            let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
            let pem = std::fs::read_to_string(data_dir.join("cert\\ca.pem")).expect("Failed to read ca.pem");
            let ca = Certificate::from_pem(pem);
            let client_cert = std::fs::read_to_string(data_dir.join("cert\\client1.pem")).expect("Failed to open Client pem");
            let client_key = std::fs::read_to_string(data_dir.join("cert\\client1.key")).expect("Failed to open client key");
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
                    println!("Error while broadcasting for {url}");
                    return;
                }
                Ok(channel) => { channel }
            };

            let mut c = proto::packet_sending_client::PacketSendingClient::new(channel);
            if transaction_op.is_none() {
                let block = block_op.unwrap();
                let mut trans: Vec<proto::Transaction> = Vec::new();
                for i in block.transactions {
                    trans.push(proto::Transaction {
                        from: i.from,
                        to: i.to,
                        amount_in: i.amount_in,
                        amount_out: i.amount_out,
                        miner_fee: i.miner_fee,
                    });
                }
                let req = proto::BlockBroadcast {
                    src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
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
                    ttl
                };
                let request = tonic::Request::new(req);
                let res = c.send_block(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to broadcast Block: {{{}}}", e);
                        return;
                    },
                    Ok(_) => {
                        info!("Send Broadcast Returned");
                        return;
                    }
                }
            } else {
                let transaction = transaction_op.unwrap();
                let req = proto::TransactionBroadcast { // Ask for a node that the server holds
                    src: auxi::gen_address(node.id.clone(), node.ip.clone(), node.port),
                    dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", ip, port).to_string()), ip.to_string(), port),
                    transaction: Some(proto::Transaction {
                        from: transaction.from,
                        to: transaction.to,
                        amount_in: transaction.amount_in,
                        amount_out: transaction.amount_out,
                        miner_fee: transaction.miner_fee,
                    }),
                    ttl,
                };
                let request = tonic::Request::new(req);
                let res = c.send_transaction(request).await;
                match res {
                    Err(e) => {
                        error!("An error has occurred while trying to broadcast Transaction: {{{}}}", e);
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