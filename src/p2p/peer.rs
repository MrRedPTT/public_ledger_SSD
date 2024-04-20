use std::sync::{Arc, Mutex};

use log::{debug, info};
use tokio::signal;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::Node;
use crate::ledger::blockchain::Blockchain;
use crate::proto::packet_sending_server::PacketSendingServer;

pub const TTL: u32 = 15; // The default ttl for the broadcast of messages

#[derive(Debug, Clone)]
pub struct Peer {
    pub node: Node,
    pub kademlia: Arc<Mutex<Kademlia>>,
    pub blockchain: Arc<Mutex<Blockchain>>
}

impl Peer {

    /// # new
    /// Creates two new instances of Peer, the
    /// server and the client which share the same kademlia attribute.
    /// They can do the same, however, the server should be used only for the init_server()
    /// given that this function will consume the object. The client will be used to initiate connections
    /// but with access to the same information (kademlia object) as the server. This share is made through
    /// [Arc<Mutex<Kademlia>>] meaning that it's thread safe.
    pub fn new(node: &Node) -> (Peer, Peer) {
        let kademlia = Arc::new(Mutex::new(Kademlia::new(node.clone())));
        let blockchain = Arc::new(Mutex::new(Blockchain::new(true, "My Name Is Mario".to_string())));
        let server = Peer {
            node: node.clone(),
            kademlia: Arc::clone(&kademlia),
            blockchain: Arc::clone(&blockchain)
        };

        let client = Peer {
            node: node.clone(),
            kademlia,
            blockchain
        };

        (server, client) // Return 2 instances of Peer that share the same kademlia object
    }

    /// # init_server
    /// Instantiates a tonic server that will listen for RPC calls. The information used,
    /// namely ip and port, is the one provided through the node.
    /// The server will be created on a different thread, and
    /// a different thread will also be created which only purpose is to listen
    /// for CTRL + C signals
    ///
    /// ### Returns
    /// As mentioned, this function will also create a second thread listening for signals.
    /// In order for the server to keep running, you need to block the main thread otherwise once
    /// it ends, it will shut down every other thread. For that, the second thread will return a receiver
    /// with which you can do something like `let _ = init_server().await`. This way, the main thread will
    /// be blocked, and the process will only terminate once a CTRL + C is detected.
    pub async fn init_server(self) -> oneshot::Receiver<()> {
        let node = self.node.clone();
        debug!("DEBUG PEER::INIT_SERVER => Creating server at {}:{}", node.ip, node.port);
        let server = Server::builder()
            .concurrency_limit_per_connection(40)
            .add_service(PacketSendingServer::new(self))
            .serve(format!("{}:{}", node.ip, node.port).parse().unwrap());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            // Wait for CTRL+C signal
            signal::ctrl_c().await.expect("failed to listen for event");
            // Send shutdown signal to the server thread
            let _ = shutdown_tx.send(());

            // Print a message indicating the server is shutting down
           info!("Shutting down server...");
        });

        tokio::spawn(async move {
            if let Err(e) = server.await {
                println!("Server error: {}", e);
            }
        });
        shutdown_rx // Channel to receive shutdown signal from the server thread
    }


}
