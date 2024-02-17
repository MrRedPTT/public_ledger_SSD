use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

pub struct Peer {
    address: String,
}

impl Peer {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    pub fn start_listener(&self) {
        let listener = TcpListener::bind(&self.address).expect("Failed to bind to address");
        println!("Listening on {}", self.address);

        for connection in listener.incoming() {
            match connection {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    thread::spawn(|| Peer::handle_client(stream));
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    }

    pub fn connect_to_peer(&self, peer_address: &str, message: &str) {
        match TcpStream::connect(peer_address) {
            Ok(mut stream) => {
                println!("Successfully connected to {}", peer_address);
                stream.write_all(message.as_bytes()).expect("Failed to send message");
                stream.flush().expect("Failed to flush");
            }
            Err(e) => println!("Failed to connect to {}: {}", peer_address, e),
        }
    }

    fn handle_client(mut stream: TcpStream) {
        let mut buffer = [0; 1024]; // Ensure buffer is large enough
        while let Ok(size) = stream.read(&mut buffer) {
            if size == 0 { // Check if the connection was closed
                break;
            }
            // Log received data for debugging
            println!("Received: {}", String::from_utf8_lossy(&buffer[..size]));

            // Echo back received data
            stream.write_all(&buffer[0..size]).expect("Failed to write to stream");
        }
        println!("Connection with {} closed.", stream.peer_addr().unwrap());
    }
}