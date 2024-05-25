extern crate core;

use std::env;
use std::io::stdin;

use crate::auction::auction::Auction;
use crate::kademlia::node::{ID_LEN, Identifier, Node};
use crate::ledger::blockchain::Blockchain;
use crate::marco::marco::Marco;
use crate::marco::transaction::Transaction;
use crate::p2p::peer::Peer;

pub mod marco;
pub mod kademlia;
pub mod ledger;
pub mod auction;

pub mod p2p;

pub mod proto {
    tonic::include_proto!("rpcpacket");
}
pub mod auxi;


#[tokio::main]
async fn main() {
    // This is here so that I can still test the project outside of docker
    let os = env::var("OS_CONF").unwrap_or_else(|_| "client".to_string());
    let mut server = env::var("EXEC_MODE").unwrap_or_else(|_| "CLIENT".to_string());
    if os == "windows" {
        let args: Vec<String> = env::args().collect();
        server = args[1].clone();
    }
    if server.to_string() == "SERVER1" {
        println!("Argument \"1\" passed, creating server...");
        test_server().await;
    } else if server.to_string() == "SERVER3" {
        println!("Argument \"3\" passed, creating server...");
        test_server_blockchain_node().await;
    } else if server.to_string() == "BOOTSTRAP" {
        println!("Bootstraping this shiiiiiiiiiiii...");
        test_bootstrap_code().await;
    } else {
        println!("Creating client...");
        test_client().await;
    }

}

async fn test_bootstrap_code() {
    println!("Initiating Bootstrap node...");
    // Bootstrap will always be at 8635
    let node = &Node::new("127.0.0.1".to_string(), 8635).unwrap();
    let (_, server) = Peer::new(node, true);
    let shutdown_rx = server.init_server().await;
    println!("Bootstrap node created!");
    println!("Listening at 127.0.0.1:8635");
    let _ = shutdown_rx.await;
}

async fn test_server_blockchain_node() {
    println!("Creating server with blockchain content");
    let node = &Node::new("127.0.0.1".to_string(), auxi::get_port().await as u32).unwrap();
    let (client, server) = Peer::new(node, false);
    let shutdown_rx = server.init_server().await;

    // Ping bootstrap node to make sure we're added to the KBucket (In a real scenario we will use bootstrap)
    let mut ping_boot = true;
    while ping_boot {
        let ping = client.ping("127.0.0.1", 8635, auxi::gen_id(format!("{}:{}", "127.0.0.1", 8635))).await;
        match ping {
            Err(_) => {},
            Ok(_) => {ping_boot = false;}
        }
    }
    client.boot().await;

    // Creating a blockchain to test the get_block RPC
    println!("Creating blockchain for testing");
    let strings = vec![
        "Alice".to_string(),
        "Bob".to_string(),
        "Carlos".to_string(),
        "Diana".to_string(),
        "Luna".to_string(),
        "Sofia".to_string(),
        "Catarina".to_string(),
        "Roberto".to_string(),
        "Gabriel".to_string(),
        "Daniel".to_string()
    ];
    let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
    let mut slash = "\\";
    if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "linux" {
        slash = "/";
    }
    let client_cert = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to open server.crt");
    let pub_key = auxi::get_public_key(client_cert);
    let mut count = 0;
    for i in 0..3 {
        for j in 0..Blockchain::MAX_TRANSACTIONS {
            println!("Was able to add to blockchain? {}", client.blockchain.lock().unwrap().add_marco(gen_transaction(strings[i+j].clone(), count), pub_key.clone()));
            count += 1;
        }
        client.blockchain.lock().unwrap().mine();
    }
    let list = client.blockchain.lock().unwrap().chain.clone();
    for i in list {
        println!("Block: id: {{{}}} hash:{}",i.index.clone(), i.hash.clone());
    }
    let list = client.blockchain.lock().unwrap().heads.get_main().clone();
    for i in list {
        println!("Block{{Head}}: id: {{{}}} hash:{}",i.index.clone(), i.hash.clone());
    }

    //
    let mut key_server_should_have = node.id.clone();
    if key_server_should_have.0[ID_LEN - 1] == 0 {
        key_server_should_have.0[ID_LEN - 1] = 1;
    } else {
        key_server_should_have.0[ID_LEN - 1] = 0;
    }
    println!("Listening at {}:{}", node.ip, node.port);

    println!("Do I have the key?: {}", !client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()).is_none());
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    println!("Do I have the key?: {}", !client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()).is_none());
    if let Some(key) = client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()) {
        println!("Key stored in server: {key}");
    }
    let _ = shutdown_rx.await;
}

async fn test_server() {
    let node = &Node::new("127.0.0.1".to_string(), auxi::get_port().await as u32).unwrap();
    let (client, server) = Peer::new(node, false);
    let shutdown_rx = server.init_server().await;

    // Ping bootstrap node to make sure we're added to the KBucket (In a real scenario we will use bootstrap)
    let mut ping_boot = true;
    while ping_boot {
        let ping = client.ping("127.0.0.1", 8635, auxi::gen_id(format!("{}:{}", "127.0.0.1", 8635))).await;
        match ping {
            Err(e) => {println!("DEBUG MAIN::TEST_SERVER => {e}")},
            Ok(_) => {ping_boot = false;}
        }
    }
    client.boot().await;

    let mut key_server_should_have = node.id.clone();
    if key_server_should_have.0[ID_LEN - 1] == 0 {
        key_server_should_have.0[ID_LEN - 1] = 1;
    } else {
        key_server_should_have.0[ID_LEN - 1] = 0;
    }

    // Add node for test in client
    client.kademlia.lock().unwrap().add_node(&Node::new("127.54.123.2".to_string(),9981).unwrap());
    println!("Listening at {}:{}", node.ip, node.port);

    println!("Do I have the key?: {}", !client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()).is_none());
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    println!("Do I have the key?: {}", !client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()).is_none());
    if let Some(key) = client.kademlia.lock().unwrap().get_value(key_server_should_have.clone()) {
        println!("Key stored in server: {key}");
    }

    let _ = shutdown_rx.await;
}

async fn test_client() {
    let auction = &Auction::new().await;
    println!("Listening at 127.0.0.1:{}", auction.client.node.port);

    let mut keys: Vec<Identifier> = Vec::new();
    let nodes_stored = auction.client.kademlia.lock().unwrap().get_all_nodes().unwrap_or(Vec::new());
    // Here we can always assume the vec is not empty
    for i in nodes_stored {
        if i.port == 8635 {continue;}
        let mut key_id = i.id.clone();
        if key_id.0[ID_LEN - 1] == 0 {
            key_id.0[ID_LEN - 1] = 1;
        } else {
            key_id.0[ID_LEN - 1] = 0;
        }
        keys.push(key_id);
    }


    println!("Get Block -> {:?}", auction.client.get_block("004048e475898274f4ab7e01aeaa2e4b60e4a7461024ee4cc91ac95a2205385483e8a8d4d13f9fa58b03c2ed2cd23b6fc26070745dcbae96166b1802ea5d7bfa".to_string()).await);
    println!("Broadcasted Transaction -> {:?}", auction.client.send_marco(gen_transaction("Testing broadcast of transaction in Client".to_string(), 2)).await);
    println!("Broadcast Block -> {:?}", auction.client.send_block(auction.client.blockchain.lock().unwrap().get_head()).await);
    println!("Result -> {:?}", auction.client.find_node(auxi::gen_id("127.0.0.2:8890".to_string())).await); // Should fail
    println!("Result -> {:?}", auction.client.find_node(auxi::gen_id("127.54.123.2:9981".to_string())).await); // Should succeed (Server1 has this node)
    for i in &keys {
        println!("Result Store -> {:?}", auction.client.store(i.clone(), "Some Random Value Server3 Should Have".to_string()).await); // Should get remote store
        println!("Result Find Key -> {:?}", auction.client.find_value(i.clone()).await); // Should succeed
    }

    auction.client.blockchain.lock().unwrap().mine();
    let list = auction.client.blockchain.lock().unwrap().chain.clone();
    for i in list {
        println!("Block{{{}}}: hash -> {}; prev_hash -> {}", i.index, i.hash.clone(), i.prev_hash.clone());
    }
    println!("Enter the hash you want to search for:");
    let stdin = stdin();
    let mut hash = String::new();
    stdin.read_line(&mut hash).expect("Failed to read line");
    if hash.ends_with('\n') {
        hash.pop();
        if hash.ends_with('\r') {
            hash.pop();
        }
    }

    println!("Get Block with hash: {} ->\n{:?}",hash.clone(), auction.client.get_block(hash).await);
    let list = auction.client.blockchain.lock().unwrap().chain.clone();
    for i in list {
        println!("Block{{{}}}: hash -> {}; prev_hash -> {}", i.index, i.hash.clone(), i.prev_hash.clone());
    }
    let list = auction.client.blockchain.lock().unwrap().heads.get_main().clone();
    for i in list {
        println!("Block{{Head}}: hash -> {}; prev_hash -> {}", i.hash.clone(), i.prev_hash.clone());
    }

}

fn gen_transaction(from: String, id: u32) -> Marco{

    let from = from;
    let to = "test".to_string() + &id.to_string();
    let out = 0.0;
    let _in = 0.0;

    Marco::from_transaction(Transaction::new(out,
                     from,
                     out-_in,
                     to))
}
