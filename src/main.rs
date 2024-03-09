extern crate core;
use std::env;
use tonic::transport::Server;
use crate::kademlia::node::Node;
use crate::p2p::peer::Peer;
use crate::proto::Address;
use crate::proto::packet_sending_server::{PacketSendingServer, PacketSending};



mod kademlia{
    pub mod test_network;
    pub mod kademlia;
    pub mod node;
    pub mod k_buckets;
    pub mod key;

    pub mod bucket;
    pub mod auxi;
}
pub mod ledger;

mod p2p{
    pub mod peer;

}

pub mod proto {
    tonic::include_proto!("rpcpacket");
}
#[tokio::main]
async fn main() {

    let node1 = Node::new("127.0.0.1".to_string(), 8888).expect("Failed to create Node1");

    let args: Vec<String> = env::args().collect();

    let server = &args[1];
    let mut server_bool: bool = false;
    if server.to_string() == "1" {
        server_bool = true;
        println!("Argument \"1\" passed, creating server...");
    }

    // Start the network listener in a separate task.
    if server_bool {
        let mut rpc = Peer::new(&node1).await.unwrap();

        // Here we are going to add some random ips
        for i in 1..=20 {
            let ip = format!("127.0.0.{}", i);
            let port = 8888 + i;
            let _ = rpc.kademlia.add_node(Node::new(ip, port).unwrap());
        }

        Server::builder()
            .concurrency_limit_per_connection(40) // Max concurrent connections to the peer (when reached, packets are placed on queues)
            .add_service(PacketSendingServer::new(rpc))
            .serve("127.0.0.1:8888".parse().unwrap())
            .await
            .unwrap();
    } else {
        let url = "http://127.0.0.1:8888";
        let mut client = proto::packet_sending_client::PacketSendingClient::connect(url).await.unwrap();

        let req = proto::PingPacket {
            src: test_fn_gen_address("127.0.0.1".to_string(), 9999),
            dst: test_fn_gen_address("127.0.0.1".to_string(), 8888)
        };

        let request = tonic::Request::new(req);
        let response = client.ping(request).await.unwrap();

        println!("Got a Pong from: {:?}", response.get_ref());

        let req2 = proto::FindNodeRequest { // Ask for a node that the server holds
            id: Node::gen_id("127.0.0.1".to_string(), 8889).0.to_vec()
        };
        let request = tonic::Request::new(req2);
        let response = client.find_node(request).await.unwrap();

        println!("Find Node Response: {:?}", response.get_ref());

        let req3 = proto::FindNodeRequest { // Ask for a node that the server does not hold
            id: Node::gen_id("127.0.04".to_string(), 8889).0.to_vec()
        };
        let request = tonic::Request::new(req3);
        let response = client.find_node(request).await.unwrap();

        println!("Find Node Response: {:?}", response.get_ref());


    }
}

fn test_fn_gen_address(ip: String, port: u32) -> Option<Address> {
    Some(proto::Address {
        ip,
        port
    })
}
