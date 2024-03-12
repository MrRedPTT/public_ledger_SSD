#[doc(inline)]
use tonic::{Request, Response, Status};
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::p2p::peer::Peer;
use crate::{proto};
use crate::kademlia::auxi;
use crate::proto::{Address, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, KNearestNodes, PingPacket, PongPacket, StoreRequest, StoreResponse};

pub struct ReqHandler {}


impl ReqHandler {

    pub async fn ping(peer: &Peer, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        let input = request.get_ref();
        println!("Test of near nodes: {:?}", peer.kademlia.lock().unwrap().get_k_nearest_to_node(peer.node.id.clone()).unwrap().len());
        if input.src.is_none() || input.dst.is_none() {
            // Abort!
        }
        let args = <Option<Address> as Clone>::clone(&input.dst).unwrap(); // Avoid Borrowing
        if args.ip != peer.node.ip || args.port != peer.node.port || args.id != peer.node.id.0.to_vec() {
            // Abort!
        }

        let node = peer.node.clone();

        let pong = PongPacket {
            src: Self::return_option(Address{id: node.id.0.to_vec(), ip: node.ip, port: node.port}),
            dst: input.clone().src
        };

        println!("Got a Ping from: {}:{}", input.clone().src.unwrap().ip, input.clone().src.unwrap().port);
        println!("Sending Pong...");

        Ok(tonic::Response::new(pong))

    }

    /// # find_node
    /// This function will handle a find_node request received by the node "server"
    /// It will do a variety of checks like verifying is dst address corresponds to the "server" node address
    /// Check if the sender node was stored in the kbucket, if it is, check if the src address corresponds to the
    /// one stored in the bucket. If it's not stored, try to add it, if the bucket is full check the last node contacted from
    /// that bucket (node on top) and see if it's alive, if it is, discard the new node and send the checked node to the back of the vec,
    /// if it's dead, remove the old node and add the sender node to the back of the Vec
    /// ## Returns
    /// This handle will check if the node is present inside the kbucket, if it is, send back a package with ResponseType 2
    /// meaning that the package contains the target node (and some default information on the other fields which should be ignored)
    /// If the node does not exist in the kbucket, send back a packet with ResponseType 1, meaning that the package contains up to
    /// k nearest nodes to the target. Finally, if anything goes wrong (except for panics), send a package with ResponseType 0, meaning
    /// a problem occurred, which will be stated in the "error" field of the message. Keep in mind that this handle function has as it's
    /// objective returning a valid package to be sent, by the tonic framework, back to the sender.
    ///
    /// More information about the package structure can be found inside proto/rpc_packet.proto
    pub async fn find_node(peer: &Peer, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
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
        let placeholder_node = Self::return_option(proto::Node { // Won't be read (used in case or error)
            id: my_node.id.0.to_vec(),
            ip: my_node.ip.clone(),
            port: peer.node.port
        });


        let lookup_src_node = peer.kademlia.lock().unwrap().get_node(Identifier::new(src_id_array));
        if lookup_src_node.is_none() {
            println!("Source node not recognized. Adding to the routing table");
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
            let response = FindNodeResponse {
                response_type: 0, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node.clone(),
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: "The supplied source is different from the one stored".to_string()
            };
            return Ok(tonic::Response::new(response));
        }

        if dst.ip != peer.node.ip || dst.port != peer.node.port || dst.id != peer.node.id.0.to_vec() {
            let response = FindNodeResponse {
                response_type: 0, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node.clone(),
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: format!("The supplied destination is invalid!\nYou passed: {}:{} id = {:?}\nExpected: {}:{} id = {:?}", dst.ip, dst.port, dst.id, peer.node.ip, peer.node.port, peer.node.id.0.to_vec()).to_string()
            };
            return Ok(tonic::Response::new(response));
        }
        println!("Node: {:?} asked about node: {:?}", request.remote_addr(), input);

        if node_id.len() != ID_LEN {
            let response = FindNodeResponse {
                response_type: 0, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node,
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: "The supplied ID has an invalid size".to_string()
            };
            return Ok(tonic::Response::new(response));
        }

        let id = Identifier::new(id_array);
        let lookup = peer.kademlia.lock().unwrap().get_node(id.clone());

        if lookup.is_none() {
            // Node not found, send k nearest nodes to
            let k_nearest = peer.kademlia.lock().unwrap().get_k_nearest_to_node(id.clone());
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
                    response_type: 1, // KNear => Send up to k near nodes to the target
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


    pub async fn find_value(peer: &Peer, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
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
                list: Self::return_option(KNearestNodes {
                    nodes: Vec::new()
                }),

                value: lookup_value.unwrap().clone(),
                error: "".to_string(),
            };

            Ok(tonic::Response::new(response))

            // Lookup for the value failed:
        } else {
            // Let's get the k nearest nodes to the value
            let nodes_lookup = mutex_guard.get_k_nearest_to_node(Identifier::new(id_array));
            // Neither the value nor the nodes were found, return error
            if nodes_lookup.is_none() {
                let response = FindValueResponse {
                    response_type: 0, // Error
                    list: Self::return_option(KNearestNodes {
                        nodes: Vec::new()
                    }),

                    value: "".to_string(),
                    error: "Neither the value nor any nodes were found".to_string(),
                };

                Ok(tonic::Response::new(response))
            } else {
                let list = nodes_lookup.unwrap();

                let mut new_list: Vec<proto::Node> = Vec::new();
                for i in list {
                    new_list.push(proto::Node { id: i.id.0.to_vec(), ip: i.ip, port: i.port });
                }

                let response = FindValueResponse {
                    response_type: 1, // ReRoute
                    list: Self::return_option(KNearestNodes {
                        nodes: new_list
                    }),

                    value: "".to_string(),
                    error: "".to_string(),
                };

                Ok(tonic::Response::new(response))
            }
        }
    }

    pub async fn store(peer: &Peer, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
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

        let mut mutex_guard = peer.kademlia.lock().unwrap();
        let nodes = mutex_guard.is_closest(&Identifier::new(id_array));
        if nodes.is_none() {
            // Means we are the closest node to the key
            mutex_guard.add_key(Identifier::new(id_array), input.value.clone());
            return if !mutex_guard.get_value(Identifier::new(id_array)).is_none() {
                let response = StoreResponse {
                    response_type: 1,
                    error: "".to_string(),
                };
                Ok(tonic::Response::new(response))
            } else {
                let response = StoreResponse {
                    response_type: 0, // Error
                    error: "Failed to store the key locally".to_string(),
                };
                Ok(tonic::Response::new(response))
            }
        } else {
            // Means we are not the closest node
            let mut thread_number = 0;
            let const_input = input.value.clone();
            for i in nodes.unwrap() {
                let url = format!("http://{}:{}", i.ip, i.port);
                let inp = const_input.clone();
                let my_node = peer.node.clone();
                let d = dst.clone(); // Since the thread will continue after the return we need a way to a hard copy of the object
                tokio::spawn(async move {
                    // Here we could use the peer.store() function
                    // However, that would imply consuming the peer object or
                    // forcing the usage of more mutexes, something we are trying to avoid
                    println!("DEBUG IN REQ_HANDLER::STORE => Thread {} initiated!", thread_number);
                    let mut c = proto::packet_sending_client::PacketSendingClient::connect(url).await;
                    if c.is_err() {
                        eprintln!("Error trying to store. Connection refused/timeout");
                        return;
                    }
                    let mut client = c.unwrap();
                    let req = StoreRequest {
                        key: id_array.to_vec(),
                        value: inp.clone(),
                        src: auxi::gen_address(my_node.id.clone(), my_node.ip.clone(), my_node.port),
                        dst: auxi::gen_address(auxi::gen_id(format!("{}:{}", d.ip.clone(), dst.port.clone()).to_string()), d.ip.clone().to_string(), dst.port),
                    };

                    let request = tonic::Request::new(req);
                    let _ = client.store(request).await.expect("Error while trying to store");
                    println!("DEBUG IN REQ_HANDLER::STORE => Thread {} terminated!", thread_number);
                });
                thread_number += 1;
            }
            // no pool.join() here since we want the response to be sent back
            // regardless if the nodes responded or not
            let response = StoreResponse {
                response_type: 2, // Forwarded the request to the k near nodes
                error: "".to_string(),
            };
            Ok(tonic::Response::new(response))

        }


    }



    fn return_option<T>(arg: T) -> Option<T> {
        Some(arg)
    }

}