use tonic::{Request, Response, Status};
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::p2p::peer::Peer;
use crate::proto;
use crate::proto::{Address, FindNodeRequest, FindNodeResponse, PingPacket, PongPacket};

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

    pub async fn find_node(peer: &Peer, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let input = request.get_ref();
        let dst =  <Option<Address> as Clone>::clone(&input.dst).unwrap(); // Avoid Borrowing
        let src =  <Option<Address> as Clone>::clone(&input.src).unwrap(); // Avoid Borrowing

        let node_id = &input.id;
        let my_node = &peer.node;
        let mut id_array: [u8; ID_LEN] = [0; ID_LEN];
        for i in 0..node_id.len() {
            id_array[i] = node_id[i];
        }
        let placeholder_node = Self::return_option(proto::Node { // Won't be read (used in case or error)
            id: my_node.id.0.to_vec(),
            ip: my_node.ip.clone(),
            port: peer.node.port
        });

        let mut kademlia_ref = &mut *peer.kademlia.lock().unwrap();

        let lookup_src_node = kademlia_ref.get_node(Identifier::new(id_array));
        if lookup_src_node.is_none() {
            println!("Source node not recognized. Adding to the routing table");
            let result = kademlia_ref.add_node(&Node::new(src.ip, src.port).unwrap());
            if !result.is_none() {
                // This means that when we tried to add the node the corresponding bucket was full
                // So the top most node was returned, now we need to check if this node is up or not
                // If it is, we can just ignore this node, if it isn't we add this new node to the kbucket

                todo!()
                // In order to avoid locking the response we might create a new thread that will deal with
                // the ping to the stored node
            }
        } else if format!("{}:{}", src.ip, src.port).to_string() != format!("{}:{}", lookup_src_node.as_ref().unwrap().ip, lookup_src_node.as_ref().unwrap().port) {
            let response = FindNodeResponse {
                response_type: 1, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node.clone(),
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: "The supplied source is different from the one stored".to_string()
            };
            return Ok(tonic::Response::new(response));
        }

        if dst.ip != peer.node.ip || dst.port != peer.node.port || dst.id != peer.node.id.0.to_vec() {
            let response = FindNodeResponse {
                response_type: 1, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node.clone(),
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: format!("The supplied destination is invalid!\nYou passed: {}:{} id = {:?}\nExpected: {}:{} id = {:?}", dst.ip, dst.port, dst.id, peer.node.ip, peer.node.port, peer.node.id.0.to_vec()).to_string()
            };
            return Ok(tonic::Response::new(response));
        }
        println!("Node: {:?} asked about node: {:?}", request.remote_addr(), input);

        if node_id.len() != ID_LEN {
            let response = FindNodeResponse {
                response_type: 1, // UNKNOWN_TYPE => Abort this communication
                node: placeholder_node,
                list: Self::return_option(proto::KNearestNodes{nodes: Vec::new()}),
                error: "The supplied ID has an invalid size".to_string()
            };
            return Ok(tonic::Response::new(response));
        }

        let id = Identifier::new(id_array);
        let lookup = kademlia_ref.get_node(id.clone());

        if lookup.is_none() {
            // Node not found, send k nearest nodes to
            let k_nearest = kademlia_ref.get_k_nearest_to_node(id.clone());
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
                    response_type: 1, // UNKNOWN_TYPE => Abort this communication
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



    fn return_option<T>(arg: T) -> Option<T> {
        Some(arg)
    }

}