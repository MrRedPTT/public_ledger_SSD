use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use log::{debug, error, info};

use crate::{auxi, proto};
use crate::kademlia::bucket::K;
use crate::kademlia::node::{ID_LEN, Node};
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::{Peer, TTL};
use crate::p2p::private::res_handler::ResHandler;

pub struct BroadCastReq {}

impl BroadCastReq {
    pub async fn send_transaction(peer: &Peer, transaction: Transaction, ttl: Option<u32>, sender: Option<Node>) {
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
                tokio::spawn(async move {
                    // Acquire a permit from the semaphore
                    let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                    for target in arg {
                        if target == Node::new("127.0.46.1".to_string(), 8935).unwrap() {
                        }
                        Self::send_transaction_request(&node, target.ip, target.port, trans.clone(), time).await;
                    }
                    drop(permit);
                })
            })
            .collect::<Vec<_>>();

        for task in tasks {
            let _ = task.await.expect("Failed to retrieve task return");
        }

        // Propagate message
        println!("Transaction propagated!");
    }

    async fn send_transaction_request(node: &Node, ip: String, port: u32, transaction: Transaction, ttl: u32) {
        let mut url = "http://".to_string();
        url += &format!("{}:{}", ip, port);
        let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;
        match c {
            Err(e) => {
                error!("An error has occurred while trying to establish a connection for transaction broadcast: {}", e);
                return;
            },
            Ok(mut client) => {
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
                        error!("An error has occurred while trying to broadcast transaction: {{{}}}", e);
                        return;
                    },
                    Ok(_) => {
                        info!("Send Broadcast Returned");
                        return;
                    }
                }
            }
        }
    }
}