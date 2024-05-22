use std::env;
use std::sync::{Arc, Mutex};

use log::{debug, info};
use sha3::{Digest, Sha3_256};
use tokio::signal;
use tokio::sync::oneshot;
use tonic::transport::{Identity, Server};

use crate::kademlia::kademlia::Kademlia;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::ledger::blockchain::Blockchain;
use crate::proto::packet_sending_server::PacketSendingServer;

pub const TTL: u32 = 15; // The default ttl for the broadcast of messages

#[derive(Debug, Clone)]
pub struct Peer {
    pub node: Node,
    pub kademlia: Arc<Mutex<Kademlia>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub bootstrap: bool,
    pub id: Identifier
}

impl Peer {

    /// # new
    /// Creates two new instances of Peer, the
    /// server and the client which share the same kademlia attribute.
    /// They can do the same, however, the server should be used only for the init_server()
    /// given that this function will consume the object. The client will be used to initiate connections
    /// but with access to the same information (kademlia object) as the server. This share is made through
    /// [Arc<Mutex<Kademlia>>] meaning that it's thread safe.
    pub fn new(node: &Node, bootstrap: bool) -> (Peer, Peer) {
        let kademlia = Arc::new(Mutex::new(Kademlia::new(node.clone())));
        let blockchain = Arc::new(Mutex::new(Blockchain::new(true, "My Name Is Mario".to_string())));
        let id = Self::gen_id(node.ip.clone(), node.port);
        let server = Peer {
            node: node.clone(),
            kademlia: Arc::clone(&kademlia),
            blockchain: Arc::clone(&blockchain),
            bootstrap,
            id: id.clone()
        };

        let client = Peer {
            node: node.clone(),
            kademlia,
            blockchain,
            bootstrap,
            id
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
        let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
        println!("Path: <{}>", data_dir.display());
        let mut slash = "\\";
        if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
            slash = "/";
        }
        let cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to open file server.crt");
        let key = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.key"))).expect("Failed to open file server.key");

        let identity = Identity::from_pem(cert, key);

        let tls = tonic::transport::ServerTlsConfig::new()
            .identity(identity);

        let server = Server::builder()
            .tls_config(tls)
            .expect("Failed to configure TLS on server")
            .concurrency_limit_per_connection(256)
            .add_service(PacketSendingServer::new(self))
            .serve(format!("{}:{}", "0.0.0.0", node.port).parse().unwrap());


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

    fn gen_id(ip: String, port: u32) -> Identifier {
        let mut slash = "/";
        if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "windows" {
            slash = "\\";
        }
        let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
        let mut pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to read ca.pem");
        pem += format!("{}:{}", ip, port).as_str();
        let mut hasher = Sha3_256::new();
        hasher.update(pem.as_bytes());

        // we know that the hash output is going to be 256 bits = 32 bytes
        let result = hasher.finalize();

        let mut hash = [0; ID_LEN];
        let mut bin_str = "".to_string();
        for b in result {
            bin_str += &format!("{:0>8b}", b); // Force the representation to pad with 0's in case there aren't 8 bits
        }

        let mut i = 0;
        assert_eq!(bin_str.len(), ID_LEN); // assert that we indeed have ID_LEN bits
        for b in bin_str.chars() {
            hash[i] = b.to_digit(2).unwrap() as u8;
            i += 1;
        }
        Identifier::new(hash)
    }

}
