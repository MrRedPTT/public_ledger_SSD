#[doc(inline)]

use std::sync::Arc;

use log::{debug, error, info};
use tonic::{Request, Response, Status};

use crate::auxi;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::p2p::peer::Peer;
use crate::p2p::private::req_handler_modules::req_handler_lookups::ReqHandler;
use crate::p2p::private::req_handler_modules::res_handler::ResHandler;
use crate::proto::{Address, PingPacket, PongPacket, StoreRequest, StoreResponse};

impl ReqHandler {
    /// # ping
    /// This function is the handler used when a [PingPacket] is received. Some verifications will be done
    /// such as validating fields, and if everything is correct, a [PongPacket] will be returned.
    ///
    /// ### Returns
    /// This function will either return a [Response<PongPacket>] or a [Status] indicating that something went wrong while
    /// handling the request or processing the response.

    pub(crate) async fn ping(peer: &Peer, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        println!("Got a Ping from => {:?}:{:?}", request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
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
        println!("Got a Store from => {:?}:{:?}", request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
        println!("Key to be stored: {}", request.get_ref().value.clone());
        let input = request.get_ref();
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
        let node_list: Vec<Node>;
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
                        if arg.1 == 8935 {
                            println!("DEBUG REQ_HANDLER::STORE -> Contacting: {}:{}", arg.0, arg.1);
                        }
                        let res = ResHandler::store(&node, arg.0, arg.1, Identifier::new(ident), val).await;
                        drop(permit);
                        res
                    })
                })
                .collect::<Vec<_>>();

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