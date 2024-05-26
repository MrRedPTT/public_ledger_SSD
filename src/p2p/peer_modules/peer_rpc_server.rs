use log::debug;
#[doc(inline)]
use log::info;
use tonic::{Request, Response, Status};

use crate::auxi;
use crate::kademlia::node::{Identifier, Node};
use crate::ledger::block::Block;
use crate::p2p::private::broadcast_api::BroadCastReq;
use crate::p2p::private::req_handler_modules::req_handler_lookups::ReqHandler;
use crate::proto::{BlockBroadcast, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse, GetBlockRequest, GetBlockResponse, PingPacket, PongPacket, StoreRequest, StoreResponse};
use crate::proto::packet_sending_server::PacketSending;

use super::super::peer::Peer;

#[tonic::async_trait]
impl PacketSending for Peer {
    /// # Ping Handler
    /// This function acts like a proxy function to the [ReqHandler::ping]
    async fn ping(&self, request: Request<PingPacket>) -> Result<Response<PongPacket>, Status> {
        let addr = request.remote_addr().unwrap().clone();
        let src = request.get_ref().src.as_ref().unwrap().clone();
        let res = ReqHandler::ping(self, request).await;
        return match res {
            Err(e) => {
                debug!("An error has occurred while receiving the Pong from {}: {}", addr, e);
                self.kademlia.lock().unwrap().risk_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
                Err(Status::aborted(e.to_string()))
            }
            Ok(pong) => {
                let node = Node::new(src.ip.clone(), src.port).unwrap();
                let add_result = self.kademlia.lock().unwrap().add_node(&node);
                if add_result.is_none() {
                    // Node was added
                    Ok(pong)
                } else {
                    let ip = add_result.clone().unwrap().ip;
                    let port = add_result.clone().unwrap().port;
                    let id = add_result.clone().unwrap().id;
                    let top_node_pong = self.ping(ip.as_ref(), port, id).await;
                    match top_node_pong {
                        Err(e) => {
                            // This means that the top node of the bucket is offline, so let's replace it with the new one
                            debug!("Error while trying to ping {}:{}: {}\nReplacing it with new node", ip, port, e);
                            self.kademlia.lock().unwrap().replace_node(&node);
                            Ok(pong)
                        }
                        Ok(_) => {
                            info!("Top node of the bucket is on, sending it to the back of the list");
                            self.kademlia.lock().unwrap().send_back(&add_result.unwrap());
                            Ok(pong)
                        }
                    }
                }
            }
        }
    }

    /// # Store Handler
    /// This function acts like a proxy function to the [ReqHandler::store],
    /// however it pings the sender before proceeding with the request (to strengthen source address spoofing resistance)
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        if self.bootstrap {
            return Err(Status::aborted("Bootstrap node. Available RPCS: {PING, FIND_NODE}".to_string()));
        }
        let id = request.get_ref().src.as_ref().unwrap().id.clone();
        let pong = self.ping(&request.get_ref().src.as_ref().unwrap().ip, request.get_ref().src.as_ref().unwrap().port, Identifier::new(id.try_into().unwrap())).await;
        let src = request.get_ref().src.as_ref().unwrap();
        match pong {
            Err(e) => {
                debug!("Tried to Ping {} back but got: {}", request.remote_addr().unwrap().to_string(), e);
                self.kademlia.lock().unwrap().risk_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
                return Err(Status::aborted(e.to_string()));
            }
            Ok(_) => {
                ReqHandler::store(self, request).await
            }
        }
    }

    /// # Find_Node Handler
    /// This function acts like a proxy function to the [ReqHandler::find_node],
    /// however it pings the sender before proceeding with the request (to strengthen source address spoofing resistance)
    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let pong = self.ping(&request.get_ref().src.as_ref().unwrap().ip, request.get_ref().src.as_ref().unwrap().port, Identifier::new(request.get_ref().src.as_ref().unwrap().id.clone().try_into().unwrap())).await;
        let src = request.get_ref().src.as_ref().unwrap();
        match pong {
            Err(e) => {
                debug!("Tried to Ping {} back but got: {}", request.remote_addr().unwrap().to_string(), e);
                self.kademlia.lock().unwrap().risk_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
                return Err(Status::aborted(e.to_string()));
            }
            Ok(_) => {
                ReqHandler::find_node(self, request).await
            }
        }
    }

    /// # Find_Value Handler
    /// This function acts like a proxy function to the [ReqHandler::find_value],
    /// however it pings the sender before proceeding with the request (to strengthen source address spoofing resistance)
    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        if self.bootstrap {
            return Err(Status::aborted("Bootstrap node. Available RPCS: {PING, FIND_NODE}".to_string()));
        }
        let pong = self.ping(&request.get_ref().src.as_ref().unwrap().ip, request.get_ref().src.as_ref().unwrap().port, Identifier::new(request.get_ref().src.as_ref().unwrap().id.clone().try_into().unwrap())).await;
        let src = request.get_ref().src.as_ref().unwrap();
        match pong {
            Err(e) => {
                debug!("Tried to Ping {} back but got: {}", request.remote_addr().unwrap().to_string(), e);
                self.kademlia.lock().unwrap().risk_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
                return Err(Status::aborted(e.to_string()));
            }
            Ok(_) => {
                ReqHandler::find_value(self, request).await
            }
        }
    }

    // ===================== block_chain Network APIs (Server Side) ============================ //
    async fn send_marco(&self, request: Request<crate::proto::MarcoBroadcast>) -> Result<Response<()>, Status> {
        if self.bootstrap {
            return Err(Status::aborted("Bootstrap node. Available RPCS: {PING, FIND_NODE}".to_string()));
        }
        // This is a broadcast so there is no need to ping back the sender
        let input = request.get_ref();
        let packed = input.marco.clone();
        let src = request.get_ref().src.as_ref().unwrap();
        self.kademlia.lock().unwrap().increment_interactions(Identifier::new(src.id.clone().try_into().unwrap()));
        if packed.is_none() {
            self.kademlia.lock().unwrap().reputation_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
            return Err(Status::invalid_argument("The provided marco is invalid"));
        }
        let unpacked = packed.unwrap();

        let transaction = auxi::transform_proto_to_marco(&unpacked);

        info!("Reveived a Marco: {:?} with TTL: {} from : {}:{}", transaction, input.ttl, request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());

        // Marco received
        let cert = input.cert.clone();
        let pub_key = auxi::get_public_key(cert);
        let (res, b) = self.blockchain.lock().unwrap().add_marco(transaction.clone(), pub_key) ;
        if !res{
            // Marco already stored or invalid
            return Ok(Response::new(()));
        }
        match b {
            None  => {},
            Some(block) => {
                println!("A new block was mined, sending it to everybody");
                self.send_block(block).await;
                return Ok(Response::new(()))
            },
        }


        if input.ttl > 1 && input.ttl <= 15 { // We also want to avoid propagating broadcast with absurd ttls (> 15)
            // Propagate
            let ttl: u32 = (input.ttl.clone() - 1).try_into().unwrap();
            BroadCastReq::broadcast(self, Some(transaction), None, Some(ttl), None, Some(request)).await;
        }
        return Ok(Response::new(()));
    }

    async fn send_block(&self, request: Request<BlockBroadcast>) -> Result<Response<()>, Status> {
        if self.bootstrap {
            return Err(Status::aborted("Bootstrap node. Available RPCS: {PING, FIND_NODE}".to_string()));
        }
        // This is a broadcast so there is no need to ping back the sender
        let input = request.get_ref();
        let packed = input.block.clone();
        let src = request.get_ref().src.as_ref().unwrap();
        self.kademlia.lock().unwrap().increment_interactions(Identifier::new(src.id.clone().try_into().unwrap()));
        if packed.is_none() {
            self.kademlia.lock().unwrap().reputation_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
            return Err(Status::invalid_argument("The provided transaction is invalid"));
        }
        let unpacked = packed.unwrap();

        let block = Block::proto_to_block(unpacked);
        info!("Reveived a Block: {:?} with TTL: {} from : {}:{}", block, input.ttl, request.get_ref().src.as_ref().unwrap().ip.clone(), request.get_ref().src.as_ref().unwrap().port.clone());
        // Block Handler
        if self.blockchain.lock().unwrap().get_block_by_hash(block.hash.clone()).is_some() {
            return Ok(Response::new(()));
        }
        let added = self.blockchain.lock().unwrap().add_block(block.clone());
        if !added {
            let _ = self.get_block(block.hash.clone()).await;
        }
        if input.ttl > 1 && input.ttl <= 15 { // We also want to avoid propagating broadcast with absurd ttls (> 15)
            // Propagate
            let ttl: u32 = input.ttl.clone() - 1;
            BroadCastReq::broadcast(self, None, Some(block), Some(ttl), Some(request), None).await;
        }
        return Ok(Response::new(()));
    }

    async fn get_block(&self, request: Request<GetBlockRequest>) -> Result<Response<GetBlockResponse>, Status> {
        if self.bootstrap {
            return Err(Status::aborted("Bootstrap node. Available RPCS: {PING, FIND_NODE}".to_string()));
        }
        let pong = self.ping(&request.get_ref().src.as_ref().unwrap().ip, request.get_ref().src.as_ref().unwrap().port, Identifier::new(request.get_ref().src.as_ref().unwrap().id.clone().try_into().unwrap())).await;
        let src = request.get_ref().src.as_ref().unwrap();
        match pong {
            Err(e) => {
                debug!("Tried to Ping {} back but got: {}", request.remote_addr().unwrap().to_string(), e);
                self.kademlia.lock().unwrap().risk_penalty(Identifier::new(src.id.clone().try_into().unwrap()));
                return Err(Status::aborted(e.to_string()));
            }
            Ok(_) => {
                ReqHandler::get_block(self, request).await
            }
        }
    }


}
