extern crate core;
use std::{env};
use crate::kademlia::node::{ID_LEN, Node};
use crate::p2p::peer::Peer;
use crate::proto::packet_sending_server::{PacketSending};



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

    let args: Vec<String> = env::args().collect();

    let server = &args[1];
    let mut server_bool: bool = false;
    if server.to_string() == "1" {
        server_bool = true;
        println!("Argument \"1\" passed, creating server...");
    }

    if server_bool {
        let rpc = Peer::new(&node1).await.unwrap();
        // Here we are going to add some random ips
        for i in 1..=255 {
            for j in 0..=255{
                let ip = format!("127.0.{}.{}", j, i);
                let port = 8888 + i + j;
                let kademlia_ref = &mut *rpc.kademlia.lock().unwrap();
                let _ = kademlia_ref.add_node(&Node::new(ip, port).unwrap());
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

        let _ = peer.ping(target_node.ip.as_ref(), target_node.port).await;
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, auxi::gen_id("127.0.0.2:8890".to_string())).await;
        let _ = peer.find_node(target_node.ip.as_ref(), target_node.port, auxi::gen_id("127.0.0.4:8889".to_string())).await;
        let _ = peer.ping("127.0.0.1", 8888).await;
        println!("{:?}", peer.ping("127.0.0.1", 7777).await);
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key".to_string())).await;
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;
        let _ = peer.store(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string()), "Some Value2".to_string()).await; // This returns a LocalStore
        let _ = peer.find_value(target_node.ip.clone(), target_node.port, auxi::gen_id("Some Key2".to_string())).await;

        let mut key = auxi::gen_id(format!("{}:{}", "127.0.0.20", 8888 +20).to_string()); // Generate an id equal to another node

        // Flip the last bit
        if key.0[ID_LEN -2 ] == 0 {
            key.0[ID_LEN -2 ] = 1;
        } else {
            key.0[ID_LEN -2 ] = 0;
        }

        let _ = peer.store(target_node.ip.clone(), target_node.port, key, "Some Value2".to_string()).await;

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
