extern crate core;

use std::env;

use crate::kademlia::node::{ID_LEN, Node};
use crate::p2p::peer::Peer;
use crate::proto::packet_sending_server::PacketSending;

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
    let rpc = Peer::new(&server_node).await.unwrap();
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

        // Start Server
        let shutdown_rx = rpc.init_server().await;
        println!("Server Loaded!");

        // In order to keep the server running, we created a new thread which is the one "holding" the main thread
        // from terminating (given the following await)
        // This new thread is only listening for CTRL + C signals, and when it detects one, it attempts to send
        // a value through the oneshot channel which, if successful, will return the value to the receiver,
        // proceeding with the execution, but given that it's the last instruction, the program will terminate

        // gui(); // Function responsible for displaying the GUI. When used the next instruction should be removed (also removing the signal thread)
        let _ = shutdown_rx.await;

    } else {
        let target_node = &node1;
        let peer = &Peer::new(node2).await.unwrap();
        let _ = peer.kademlia.lock().unwrap().add_node(target_node); // Add server node
        for i in 1..15 {
            let _ = peer.kademlia.lock().unwrap().add_node(&Node::new(format!("127.0.0.{}", i), 8888+i).unwrap()); // Add random nodes
        }

        let _ = peer.ping(target_node.ip.as_ref(), target_node.port).await;
        let mut key_server_should_have = auxi::gen_id(format!("{}:{}", "127.0.46.1", 8935).to_string());

        // Flip the last bit
        if key_server_should_have.0[ID_LEN - 1] == 0 {
            key_server_should_have.0[ID_LEN - 1] = 1;
        } else {
            key_server_should_have.0[ID_LEN - 1] = 0;
        }
        println!("Distance server:{}\nDistance Key:{}", auxi::xor_distance(&auxi::gen_id(format!("{}:{}", "127.0.46.1", 8935).to_string()), &node2.id.clone()), auxi::xor_distance(&key_server_should_have.clone(), &node2.id.clone()));
        println!("{}\n{}", auxi::vec_u8_to_string(key_server_should_have.clone()), auxi::vec_u8_to_string(auxi::gen_id(format!("{}:{}", "127.0.46.1", 8935).to_string())));
        println!("Ping Server3 -> {:?}", peer.ping(&node3.ip, node3.port).await);
        println!("Result -> {:?}", peer.find_node(auxi::gen_id("127.0.0.2:8890".to_string()), None, None).await);
        println!("Result -> {:?}", peer.find_node(auxi::gen_id("127.54.123.2:9981".to_string()), None, None).await);
        println!("Result -> {:?}", peer.store(key_server_should_have.clone(), "Some Random Value Server3 Should Have".to_string()).await);
        println!("Result -> {:?}", peer.find_value(key_server_should_have, None, None).await);

        /*
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, auxi::gen_id("127.0.0.4:8889".to_string())).await;
        let _ = peer.ping("127.0.0.1", 8888).await;
        println!("{:?}", peer.ping("127.0.0.1", 7777).await);
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key".to_string())).await;
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;
        let _ = peer.store(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string()), "Some Value2".to_string()).await; // This returns a LocalStore
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;

        let mut key = auxi::gen_id(format!("{}:{}", "127.0.0.20", 8888 +20).to_string()); // Generate an id equal to another node
        let mut key_server_should_have = auxi::gen_id(format!("{}:{}", "127.0.0.1", 8888).to_string());

        // Flip the last bit
        if key.0[ID_LEN -2 ] == 0 {
            key.0[ID_LEN -2 ] = 1;
        } else {
            key.0[ID_LEN -2 ] = 0;
        }

        let _ = peer.store(target_node.ip.clone(), target_node.port, key, "Some Value2".to_string()).await;
        let _ = peer.store(target_node.ip.clone(), target_node.port, key_server_should_have.clone(), "Value Server Should Have".to_string()).await;
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, key_server_should_have.clone()).await;
        */
    }
}

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
