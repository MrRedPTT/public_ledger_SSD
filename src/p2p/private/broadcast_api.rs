use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info};
use tokio::time::error::Elapsed;
use tonic::Request;
use tonic::transport::{Channel, Error};

use crate::{auxi, proto};
use crate::kademlia::bucket::K;
use crate::kademlia::node::{ID_LEN, Node};
use crate::ledger::block::Block;
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::{Peer, TTL};
use crate::p2p::private::req_handler_modules::res_handler::ResHandler;
use crate::proto::packet_sending_client::PacketSendingClient;

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


        let mut arguments: Vec<(Vec<Node>)> = Vec::new();
        if sub_vectors.len() > 0 {
            for i in &sub_vectors{
                arguments.push(i.clone())
            }
        }

        let mut size = 0;
        for s in &sub_vectors {
            size += s.len();
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
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);
        // Set the timeout duration
        let timeout_duration = Duration::from_secs(5); // 5 seconds

        // Wrap the connect call with a timeout
        let c = tokio::time::timeout(timeout_duration, async {
            // Establish the gRPC connection
            proto::packet_sending_client::PacketSendingClient::connect(url).await
        }).await;

        match c {
            Ok(Err(e)) => {
                error!("An error has occurred while trying to establish a connection for transaction broadcast: {}", e);
                return;
            },
            Ok(Ok(mut client)) => {
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
                    let res = client.send_block(request).await;
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
                    let res = client.send_transaction(request).await;
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
            }
            Err(_) => {
                error!("Connection Timeout");
                return;
            }
        }
    }

}