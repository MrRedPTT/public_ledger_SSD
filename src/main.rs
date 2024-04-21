extern crate core;

use std::env;
use std::io::stdin;
use std::sync::{Arc, RwLock};

use crate::kademlia::node::{ID_LEN, Node};
use crate::ledger::block::Block;
use crate::ledger::blockchain::Blockchain;
use crate::ledger::transaction::Transaction;
use crate::p2p::peer::Peer;

pub mod kademlia;
pub mod ledger;

pub mod p2p;

pub mod proto {
    tonic::include_proto!("rpcpacket");
}

pub mod ledger_gui;
pub mod auxi;


#[tokio::main]
async fn main() {

    let node1 = Node::new("127.0.0.1".to_string(), 8888).expect("Failed to create Node1");
    let node2 = &Node::new("127.0.0.1".to_string(), 9999).expect("Failed to create Node2");
    let node3 = &Node::new("127.0.46.1".to_string(), 8935).expect("Failed to create Node2");
    let mut server_node = node1.clone();

    let args: Vec<String> = env::args().collect();

    let server = &args[1];
    let mut server_bool: bool = false;
    if server.to_string() == "1" {
        server_bool = true;
        println!("Argument \"1\" passed, creating server...");
    } else if server.to_string() == "3" {
        server_bool = true;
        println!("Argument \"3\" passed, creating server...");
        server_node = node3.clone();
    }
    let (rpc, client) = Peer::new(&server_node);
    if server.to_string() == "3" {let _ = rpc.kademlia.lock().unwrap().add_node(&Node::new("127.54.123.2".to_string(),9981).unwrap());}

    if server_bool {

        // Here we are going to add some random ips
        let _ = rpc.kademlia.lock().unwrap().add_node(&Node::new("127.0.46.1".to_string(), 8935).unwrap()); // Add server 3
        for i in 1..=255 {
            for j in 0..=255{
                let ip = format!("127.0.{}.{}", j, i);
                let port = 8888 + i + j;
                let _ = rpc.kademlia.lock().unwrap().add_node(&Node::new(ip, port).unwrap());
            }
        }
        // Add a key value to the hashmap to be looked up
        let _ = rpc.kademlia.lock().unwrap().add_key(auxi::gen_id("Some Key".to_string()), "Some Value".to_string());


        if server.to_string() == "3" {
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
            for i in 0..6 {
                for j in 0..3 {
                    client.blockchain.lock().unwrap().add_transaction(gen_transaction(strings[i+j].clone()));
                }
                client.blockchain.lock().unwrap().mine();
            }
            let list = client.blockchain.lock().unwrap().chain.clone();
            for i in list {
                println!("Block: id: {{{}}} hash:{}",i.index.clone(), i.hash.clone());
            }
        }

        // Start Server
        let shutdown_rx = rpc.init_server().await;
        println!("Server Loaded!");

        // In order to keep the server running, we created a new thread which is the one "holding" the main thread
        // from terminating (given the following await)
        // This new thread is only listening for CTRL + C signals, and when it detects one, it attempts to send
        // a value through the oneshot channel which, if successful, will return the value to the receiver,
        // proceeding with the execution, but given that it's the last instruction, the program will terminate
        let mut key_server1_should_have = node1.id.clone();
        let mut key_server3_should_have = node3.id.clone();
        if key_server1_should_have.0[ID_LEN - 1] == 0 {
            key_server1_should_have.0[ID_LEN - 1] = 1;
        } else {
            key_server1_should_have.0[ID_LEN - 1] = 0;
        }
        if key_server3_should_have.0[ID_LEN - 1] == 0 {
            key_server3_should_have.0[ID_LEN - 1] = 1;
        } else {
            key_server3_should_have.0[ID_LEN - 1] = 0;
        }

        let test_block_events = Blockchain::new(false, "BlockHey".to_string());
        let observer = Arc::new(RwLock::new(test_block_events));
        let blockchainclient = Arc::clone(&observer);
        let client1 = Arc::new(RwLock::new(client));
        let client2 = Arc::clone(&client1);

        println!("Do I have the key1?: {}", !client2.read().unwrap().kademlia.lock().unwrap().get_value(key_server1_should_have.clone()).is_none());
        println!("Do I have the key3?: {}", !client2.read().unwrap().kademlia.lock().unwrap().get_value(key_server3_should_have.clone()).is_none());
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        println!("Do I have the key1?: {}", !client2.read().unwrap().kademlia.lock().unwrap().get_value(key_server1_should_have.clone()).is_none());
        println!("Do I have the key3?: {}", !client2.read().unwrap().kademlia.lock().unwrap().get_value(key_server3_should_have.clone()).is_none());
        println!("Do I have the Block/Transaction?: {:?}", blockchainclient.read().unwrap().chain);
        let _transaction = Transaction {
            from: "Transaction Mined".to_string(),
            to: "".to_string(),
            amount_in: 0.0,
            amount_out: 0.0,
            miner_fee: 0.0,
        };


        // gui(); // Function responsible for displaying the GUI. When used the next instruction should be removed (also removing the signal thread)
        let _ = shutdown_rx.await;

    } else {
        let target_node = &node1;
        let (peer, _client) = &Peer::new(node2);
        let _ = peer.kademlia.lock().unwrap().add_node(target_node); // Add server node
        for i in 1..15 {
            let _ = peer.kademlia.lock().unwrap().add_node(&Node::new(format!("127.0.0.{}", i), 8888+i).unwrap()); // Add random nodes
        }

        let mut key_server3_should_have = auxi::gen_id(format!("{}:{}", "127.0.46.1", 8935).to_string());
        let mut key_server1_should_have = node1.id.clone();

        // Flip the last bit
        if key_server3_should_have.0[ID_LEN - 1] == 0 {
            key_server3_should_have.0[ID_LEN - 1] = 1;
        } else {
            key_server3_should_have.0[ID_LEN - 1] = 0;
        }

        if key_server1_should_have.0[ID_LEN - 1] == 0 {
            key_server1_should_have.0[ID_LEN - 1] = 1;
        } else {
            key_server1_should_have.0[ID_LEN - 1] = 0;
        }

        let _transaction = Transaction {
            from: "Test Transaction Observer".to_string(),
            to: "".to_string(),
            amount_in: 0.0,
            amount_out: 0.0,
            miner_fee: 0.0,
        };
        
        let _block = Block::new(1, "ahsahsahsa".to_string(), 1, "jose".to_string(), 0.0);

        let server = peer.clone();
        let _ = server.init_server().await;

        println!("Get Block -> {:?}", peer.get_block("004048e475898274f4ab7e01aeaa2e4b60e4a7461024ee4cc91ac95a2205385483e8a8d4d13f9fa58b03c2ed2cd23b6fc26070745dcbae96166b1802ea5d7bfa".to_string()).await);
        //println!("Broadcasted Transaction -> {:?}", peer.send_transaction(_transaction).await);
        //println!("Broadcast Block -> {:?}", peer.send_block(_block).await);
        //println!("Ping Server1 -> {:?}", peer.ping(&node1.ip, node1.port, node1.id.clone()).await);
        //println!("Ping Server3 -> {:?}", peer.ping(&node3.ip, node3.port, node3.id.clone()).await);
        //println!("Result -> {:?}", peer.find_node(auxi::gen_id("127.0.0.2:8890".to_string())).await);
        //println!("Result -> {:?}", peer.find_node(auxi::gen_id("127.54.123.2:9981".to_string())).await);
        println!("Result Store Key3 -> {:?}", peer.store(key_server3_should_have.clone(), "Some Random Value Server3 Should Have".to_string()).await);
        println!("Result Find Key3 -> {:?}", peer.find_value(key_server3_should_have).await);
        client.blockchain.lock().unwrap().mine();
        let list = client.blockchain.lock().unwrap().chain.clone();
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
        println!("Get Block with hash: {} ->\n{:?}",hash.clone(), peer.get_block(hash).await);
        //println!("Result Store Key1 -> {:?}", peer.store(key_server1_should_have.clone(), "Some Random Value Server1 Should Have".to_string()).await);
        //println!("Result Find Key 1-> {:?}", peer.find_value(key_server1_should_have).await);
        /*let trust_scores = peer.kademlia.lock().unwrap().get_all_trust_scores();
        for mut i in trust_scores {
            println!("Node: {}:{} -> TrustScore: {}", i.0.ip, i.0.port, i.1.get_score());
        }*/

    }
}

fn gen_transaction(from: String) -> Transaction {

    let from = from;
    let to = "test".to_string();
    let out = 0.0;
    let _in = 0.0;

    Transaction::new(out,
                     from,
                     out-_in,
                     to)
}



/*
#[cfg(not(target_arch = "wasm32"))]
fn gui() -> eframe::Result<()>{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Epstein Auction",
        native_options,
        Box::new(|cc| Box::new(ledger_gui::TemplateApp::new(cc))),
    )
}
*/