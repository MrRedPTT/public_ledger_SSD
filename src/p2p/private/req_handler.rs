use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use log::{debug, error, info};
#[doc(inline)]
use tonic::{Request, Response, Status};

use crate::{auxi, kademlia, proto};
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::p2p::peer::Peer;
use crate::p2p::private::res_handler::ResHandler;
use crate::proto::{Address, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, KNearestNodes, PingPacket, PongPacket, StoreRequest, StoreResponse};

/// # Request Handler
/// This struct is where we define the handlers for all possible gRPC requests.
/// Note that this function is called by the [proto::packet_sending_server::PacketSending] trait in Peer,
/// therefore, this function should not be called directly but by the tonic
/// server instance when a [Request] is received.
/// You can see more information about the Request/Response format in the `proto/rpc_packet.proto` file.
pub(crate) struct ReqHandler {}


impl ReqHandler {

    /// # ping
    /// This function is the handler used when a [PingPacket] is received. Some verifications will be done
    /// such as validating fields, and if everything is correct, a [PongPacket] will be returned.
    ///
    /// ### Returns
    /// This function will either return a [Response<PongPacket>] or a [Status] indicating that something went wrong while
    /// handling the request or processing the response.

    pub(crate) async fn ping(peer: &Peer, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        println!("Got a Ping from => {:?}", request.remote_addr().unwrap());
        let input = request.get_ref();
        if input.src.is_none() || input.dst.is_none() {
            return Err(Status::invalid_argument("Source and/or destination not found"));
        }
        let args = <Option<Address> as Clone>::clone(&input.dst).unwrap(); // Avoid Borrowing
        if args.ip != peer.node.ip || args.port != peer.node.port || args.id != peer.node.id.0.to_vec() {
            return Err(Status::invalid_argument("Node provided in destination does not match this node"))
        }

        let node = peer.node.clone();

        let pong = PongPacket {
            src: auxi::return_option(Address{id: node.id.0.to_vec(), ip: node.ip, port: node.port}),
            dst: input.clone().src,
            rand_id: input.clone().rand_id
        };

        info!("Got a Ping from: {}:{}", input.clone().src.unwrap().ip, input.clone().src.unwrap().port);
        info!("Sending Pong...");

        Ok(tonic::Response::new(pong))

    }

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
        println!("Got a Find Node from => {:?}", request.remote_addr().unwrap());
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

                let res = peer.ping(&src.ip.clone(), src.port).await;
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
        println!("Got a Find Value from => {:?}", request.remote_addr().unwrap());
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

    /// # store
    /// This handler has a major difference in the way it behaves when compared with the previous ones.
    /// The goal of a store gRPC call is to instruct a node(s) to store a (Key ([Identifier]), Value ([String])), however
    /// if the receiver node detects that it's not the closest node to the key it will forward the request to the k nearest nodes, otherwise,
    /// it will store the key. The major difference here is that the receiver node is the onde responsible for the forwarding and not the sender.
    ///
    /// ### Returns
    /// Just like the previous requests, this handler will either return a [Status] in case something goes wrong with the request
    /// or a [StoreResponse] which contains 2 possible states, either a `LocalStore` which indicates that the node was stored at in
    /// the receiver, or `Forwarding` meaning that the receiver node has forwarded the request to the nodes it deems closer to the key.
    ///
    /// #### Important Information
    /// In order to avoid blocking the sender for too much time, the forwarding is performed within threads, meaning that if a receiver node
    /// deems that it's not the closest node and should forward the request, it will send those requests parallel to the response, so even if none of the nodes
    /// respond the sender will be under the impression that the key was dealt with.
    pub(crate) async fn store(peer: &Peer, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        // Procedure:
        // Check if our node is the closest to the key
        // If it is, store the key and send back a Response informing of the store
        // If it isn't, send the key to all the k closest node and send back a response about the forwarding
        //
        // Challenges:
        // Assuming we need to forward the requests we are faced with the following choice:
        // Waiting for all (or at least 1) to respond (either LocalStore or Forwarding) and then return the result
        // to the original sender, this may lock the sender but guarantees that we can inform the sender if the value was not stored
        // (none of the nodes are up or all of the responses were Error)
        // OR
        // Sending the requests on a different thread and simply returning the Forwarding result to the sender.
        //
        // We decided to go with the second option given that the first would require the 1st node to wait for the 2nd, the 2nd for the 3rd,
        // the 3rd for the 4th and so on. Which, in a big network would become very problematic
        println!("Got a Store from => {:?}", request.remote_addr().unwrap());
        let input = request.get_ref();
        let dst =  <Option<Address> as Clone>::clone(&input.dst).unwrap(); // Avoid Borrowing
        let src =  &<Option<Address> as Clone>::clone(&input.src).unwrap(); // Avoid Borrowing

        let key = &input.key;
        let mut id_array: [u8; ID_LEN] = [0; ID_LEN];
        let mut src_id_array: [u8; ID_LEN] = [0; ID_LEN];

        // Get the id's into an array so that we can generate Identities
        for i in 0..key.len() {
            id_array[i] = key[i];
            src_id_array[i] = src.id[i];
        }

        //let mut mutex_guard = peer.kademlia.lock().unwrap();
        let nodes = peer.kademlia.lock().unwrap().is_closest(&Identifier::new(id_array));
        let mut node_list: Vec<Node> = Vec::new();
        if nodes.is_none() {
            // Means we are the closest node to the key
            peer.kademlia.lock().unwrap().add_key(Identifier::new(id_array), input.value.clone());
            return if !peer.kademlia.lock().unwrap().get_value(Identifier::new(id_array)).is_none() {
                let response = StoreResponse {
                    response_type: 1
                };
                Ok(tonic::Response::new(response))
            } else {
                return Err(Status::internal("Failed to store the key locally"));
            }
        } else {
            // Means we are not the closest node
            let const_input = input.value.clone();

            let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Limit the amount of threads

            let mut arguments: Vec<(String, u32)> = Vec::new();
            if nodes.is_none() {
                return Err(tonic::Status::internal("Failed to find any nodes to send the request to"));
            } else {
                node_list = nodes.unwrap();
                for i in &node_list{
                    debug!("DEBUGG IN PEER::STORE -> Peers discovered: {}:{}", i.ip, i.port);
                    arguments.push((i.ip.clone(), i.port))
                }
            }

            let value = input.clone().value;
            // Process tasks concurrently using Tokio
            let tasks = arguments.into_iter()
                .map(|arg| {
                    let semaphore = semaphore.clone();
                    let node = peer.node.clone();
                    let ident = id_array.clone();
                    let val = value.clone();
                    tokio::spawn(async move {
                        // Acquire a permit from the semaphore
                        let permit = semaphore.acquire().await.expect("Failed to acquire permit");
                        debug!("DEBUG REQ_HANDLER::STORE -> Contacting: {}:{}", arg.0, arg.1);
                        let res = ResHandler::store(&node, arg.0, arg.1, Identifier::new(ident), val).await;
                        drop(permit);
                        res
                    })
                })
                .collect::<Vec<_>>();

            let mut errors: Vec<Node> = Vec::new();
            let mut type_of_return = 0;
            let mut counter: usize = 0;
            for task in tasks {
                let result = task.await.expect("Failed to retrieve task result");
                match result {
                    Err(e) => {
                        error!("Error found: {}", e);
                    }
                    Ok(res) => {
                        let resp_type = res.get_ref().response_type;
                        if resp_type == 1 || resp_type == 2 {
                            debug!("DEBUG PEER::STORE -> The node stored it");
                            type_of_return = 2; // The node stored it (or someone else along the line) so we need to return RemoteStore
                            break;
                        }
                    }
                }
                // Move the node we just contacted to the back of the list
                let _ = peer.kademlia.lock().unwrap().send_back_specific_node(&node_list[counter]);
                counter += 1;
            }

            let response = StoreResponse {
                response_type: type_of_return // Return if someone else along the line stored it or not
            };
            Ok(tonic::Response::new(response))

        }


    }

}