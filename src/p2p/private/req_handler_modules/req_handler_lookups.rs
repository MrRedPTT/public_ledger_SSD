#[doc(inline)]

use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use log::{debug, error, info};
use tonic::{Request, Response, Status};

use crate::{auxi, kademlia, proto};
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::ledger::block::Block;
use crate::ledger::blockchain::Blockchain;
use crate::p2p::peer::Peer;
use crate::proto::{Address, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, GetBlockRequest, GetBlockResponse, KNearestNodes, PingPacket, PongPacket, StoreRequest, StoreResponse};

/// # Request Handler
/// This struct is where we define the handlers for all possible gRPC requests.
/// Note that this function is called by the [proto::packet_sending_server::PacketSending] trait in Peer,
/// therefore, this function should not be called directly but by the tonic
/// server instance when a [Request] is received.
/// You can see more information about the Request/Response format in the `proto/rpc_packet.proto` file.
pub struct ReqHandler {}


impl ReqHandler {

    /// # find_node
    /// This function will handle a [FindNodeRequest] received by the tonic server.
    /// It will do a variety of checks like verifying if the dst address corresponds to the receiver node address
    /// Check if the sender node was stored in the [KBucket]([kademlia::k_buckets::KBucket]), if it is, check if the src address corresponds to the
    /// one stored in the bucket. If it's not stored, try to add it, if the bucket is full check the last node contacted from
    /// that bucket (node on top) and see if it's alive, if it is, discard the new node and send the checked node to the back of the vec,
    /// if it's dead, remove the old node and add the sender node to the back of the Vec
    /// ## Returns
    /// This handler will check if the node is present inside the [KBucket]([kademlia::k_buckets::KBucket]), if it is, send back a package ([Response<FindNodeResponse>]) with ResponseType 2
    /// meaning that the package contains the target node (and some default information on the other fields which should be ignored).
    /// If the node does not exist in the [KBucket]([kademlia::k_buckets::KBucket]), send back a packet with ResponseType 1, meaning that the package contains up to
    /// k nearest nodes to the target. Finally, if anything goes wrong, a [Status] will be returned back.
    ///
    pub(crate) async fn find_node(peer: &Peer, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        println!("Got a Find Node from => {:?}:{:?}", request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
        let input = request.get_ref();
        let dst =  <Option<Address> as Clone>::clone(&input.dst).unwrap(); // Avoid Borrowing
        let src =  &<Option<Address> as Clone>::clone(&input.src).unwrap(); // Avoid Borrowing

        let node_id = &input.id;
        let my_node = &peer.node;
        let mut id_array: [u8; ID_LEN] = [0; ID_LEN];
        let mut src_id_array: [u8; ID_LEN] = [0; ID_LEN];


        for i in 0..node_id.len() {
            id_array[i] = node_id[i];
            src_id_array[i] = src.id[i];
        }
        let placeholder_node = auxi::return_option(proto::Node { // Won't be read (used just to fill in field)
            id: my_node.id.0.to_vec(),
            ip: my_node.ip.clone(),
            port: peer.node.port
        });


        let lookup_src_node = peer.kademlia.lock().unwrap().get_node(Identifier::new(src_id_array));
        if lookup_src_node.is_none() {
            info!("Source node not recognized. Adding to the routing table");
            let result = peer.kademlia.lock().unwrap().add_node(&Node::new(src.ip.clone(), src.port).unwrap());
            if !result.is_none() {
                // This means that when we tried to add the node the corresponding bucket was full
                // So the top most node was returned, now we need to check if this node is up or not
                // If it is, we can just ignore this node, if it isn't we add this new node to the kbucket
                // In order to avoid locking the response we might create a new thread that will deal with
                // the ping to the stored node

                let res = peer.ping(&src.ip.clone(), src.port, Identifier::new(src.id.clone().try_into().unwrap())).await;
                match res{
                    Err(_) => {
                        // This means we couldn't contact the node on top of the list
                        // Therefore let's substitute it
                        let new_node = Node::new(src.ip.clone(), src.port);
                        if !new_node.is_none() {
                            peer.kademlia.lock().unwrap().replace_node(&new_node.unwrap());
                        }
                    }
                    // If the node is alive send it to the back of the list
                    Ok(_) => {
                        let new_node = Node::new(src.ip.clone(), src.port);
                        if !new_node.is_none() {
                            // The new node is passed as a way to calculate which bucket we
                            // will be acting upon
                            peer.kademlia.lock().unwrap().send_back(&new_node.unwrap());
                        }
                    }
                }

            }
        } else if format!("{}:{}", src.ip, src.port).to_string() != format!("{}:{}", lookup_src_node.as_ref().unwrap().ip, lookup_src_node.as_ref().unwrap().port) {
            return Err(Status::invalid_argument("The supplied source is different from the one stored"))
        }

        if dst.ip != peer.node.ip || dst.port != peer.node.port || dst.id != peer.node.id.0.to_vec() {
            return Err(Status::invalid_argument("The supplied destination is invalid!"));
        }
        info!("Node: {:?} asked about node: {:?}", request.remote_addr(), input);

        if node_id.len() != ID_LEN {
            return Err(Status::invalid_argument("The supplied ID has an invalid size"));
        }

        let id = Identifier::new(id_array);
        let lookup = peer.kademlia.lock().unwrap().get_node(id.clone());

        if lookup.is_none() {
            // Node not found, send k nearest nodes to
            let k_nearest = peer.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone());
            return if k_nearest.is_none() {
                return Err(Status::not_found("Neither the target node or it's nearest nodes were found"));
            } else {
                let mut list: Vec<proto::Node> = Vec::new();
                // The type cant be our definition of node but proto::Node
                // Therefore we need to create instances of those and place them inside a new vector
                for i in k_nearest.unwrap() {
                    list.push(proto::Node{id:i.id.0.to_vec(), ip:i.ip, port:i.port});
                }
                let response = FindNodeResponse {
                    response_type: 1, // KNear => Send up to k near nodes to the target
                    node: placeholder_node,
                    list: auxi::return_option(proto::KNearestNodes{nodes: list})
                };
                Ok(tonic::Response::new(response))
            }
        } else {
            // Node found, send it back
            let target_node = lookup.unwrap();
            let response = FindNodeResponse {
                response_type: 2, // Found the target node
                node: auxi::return_option(proto::Node{id: target_node.id.0.to_vec(), ip: target_node.ip, port: target_node.port}),
                list: auxi::return_option(proto::KNearestNodes{nodes: Vec::new()}) // Won't be read
            };
            Ok(tonic::Response::new(response))
        }

    }


    /// # find_value
    /// This handler is similar to the [find_node](ReqHandler::find_node). When we receive a [FindValueRequest], we will validate its fields
    /// and then check if we're currently storing the queried value and either way we send back a [FindValueResponse].
    ///
    /// ### Returns
    /// If we were able to find the value within our own [Kademlia](kademlia::kademlia::Kademlia) se send back a [Response<FindValueResponse>] with the ResponseType 2, meaning
    /// that the value was found and will be stored inside the packet in the `value` field (All other fields should be ignored). If we couldn't find the value,
    /// we still send back a packet however with ResponseType 1, indicating that the `value` field should be ignored and that the [KNearest](KNearestNodes) contains the nearest nodes (a maximum of `k`)
    /// to the target ID. If anything goes wrong, a [Status] will be returned indicating either a request or response related problem.
    ///
    pub(crate) async fn find_value(peer: &Peer, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        println!("Got a Find_value from => {:?}:{:?}", request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
        let input = request.get_ref();
        let src =  &<Option<Address> as Clone>::clone(&input.src).unwrap(); // Avoid Borrowing

        let value_id = &input.value_id;
        let mut id_array: [u8; ID_LEN] = [0; ID_LEN];
        let mut src_id_array: [u8; ID_LEN] = [0; ID_LEN];

        // Get the id's into an array so that we can generate Identities
        for i in 0..value_id.len() {
            id_array[i] = value_id[i];
            src_id_array[i] = src.id[i];
        }

        let mutex_guard = peer.kademlia.lock().unwrap();
        let lookup_value = mutex_guard.get_value(Identifier::new(id_array));

        // Lookup found:
        return if !lookup_value.is_none() {
            let response = FindValueResponse {
                response_type: 2, // Returning Value
                list: auxi::return_option(KNearestNodes {
                    nodes: Vec::new()
                }),

                value: lookup_value.unwrap().clone()
            };

            Ok(tonic::Response::new(response))

            // Lookup for the value failed:
        } else {
            // Let's get the k nearest nodes to the value
            let nodes_lookup = mutex_guard.get_k_nearest_to_node(Identifier::new(id_array));
            // Neither the value nor the nodes were found, return error
            if nodes_lookup.is_none() {
                return Err(Status::not_found("Neither the value nor any nodes were found"));
            } else {
                let list = nodes_lookup.unwrap();

                let mut new_list: Vec<proto::Node> = Vec::new();
                for i in list {
                    new_list.push(proto::Node { id: i.id.0.to_vec(), ip: i.ip, port: i.port });
                }

                let response = FindValueResponse {
                    response_type: 1, // ReRoute
                    list: auxi::return_option(KNearestNodes {
                        nodes: new_list
                    }),

                    value: "".to_string()
                };

                Ok(tonic::Response::new(response))
            }
        }
    }

    pub(crate) async fn get_block(peer: &Peer, request: Request<GetBlockRequest>) -> Result<Response<GetBlockResponse>, Status> {
        println!("Got a Get_Block from => {:?}:{:?}", request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
        let input = request.get_ref();

        let block_requested = input.id.clone();

        // ======================== TESTING WHILE THE API IS NOT READY ================================== //
        let mut server = "server1".to_string();
        if peer.node.port != 8888 {
            server = "server3".to_string();
        }
        // ======================== TESTING WHILE THE API IS NOT READY ================================== //

        if let Some(block) = peer.blockchain.lock().unwrap().dummy_get_block(server) {
            // Got the block
            println!("Returning dummy block");
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
            let res = GetBlockResponse {
                response_type: 2,
                list: None,
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
                })
            };
            return Ok(tonic::Response::new(res));
        } else {
            // No node found let's return the closest nodes
            if let Some(reroute) = peer.kademlia.lock().unwrap().get_k_nodes_new_distance() {
                let mut list: Vec<proto::Node> = Vec::new();
                for node in reroute {
                    list.push(proto::Node {
                        id: node.id.0.to_vec().clone(),
                        ip: node.ip.clone(),
                        port: node.port.clone(),
                    })
                }
                let res = GetBlockResponse {
                    response_type: 2,
                    list: Some(KNearestNodes {
                        nodes: list,
                    }),
                    block: None
                };
                return Ok(tonic::Response::new(res));
            } else {
                error!("No Nodes found");
            }
        }

        return Err(Status::not_found("No Block Found"));
    }


}