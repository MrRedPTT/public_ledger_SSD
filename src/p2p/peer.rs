use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
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

    pub fn connect_to_peer(&self, peer_address: &str, messages: Vec<&str>) {
        match TcpStream::connect(peer_address) {
            Ok(mut stream) => {
                println!("Successfully connected to {}", peer_address);
                for message in messages {
                    println!("Sending: {}", message);
                    // Send each message with a length prefix for consistency
                    let message_len = (message.len() as u32).to_be_bytes();
                    stream.write_all(&message_len).expect("Failed to write message length");
                    stream.write_all(message.as_bytes()).expect("Failed to send message");
                    stream.flush().expect("Failed to flush");

                    // Wait for an acknowledgment using read_message
                    match read_message(&mut stream) {
                        Ok(response) => {
                            let response_str = String::from_utf8_lossy(&response);
                            println!("Acknowledgment received: {}", response_str);
                            if message == "BYE" {
                                println!("Termination message sent, closing connection.");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read acknowledgment: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => println!("Failed to connect to {}: {}", peer_address, e),
        }
    }

    fn handle_client(mut stream: TcpStream) {
        loop {
            match read_message(&mut stream) {
                Ok(message) => {
                    if message.is_empty() {
                        // Assuming an empty message signifies the client has closed the connection
                        println!("Client disconnected.");
                        break;
                    }
                    // Process the message (e.g., log it, echo back)
                    println!("Received: {}", String::from_utf8_lossy(&message));

                    // Echo back the message as an acknowledgment
                    let mut length_prefix = (message.len() as u32).to_be_bytes();
                    stream.write_all(&length_prefix).expect("Failed to write length prefix");
                    stream.write_all(&message).expect("Failed to write message");
                },
                Err(e) => {
                    eprintln!("Failed to read message: {}", e);
                    break;
                }
            }
        }
        println!("Connection with {} closed.", stream.peer_addr().unwrap());
    }


}
fn read_message(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut length_buf = [0; 4]; // Assuming a 4-byte length prefix
    stream.read_exact(&mut length_buf)?;
    let msg_length = u32::from_be_bytes(length_buf) as usize;

    let mut message_buf = vec![0; msg_length];
    stream.read_exact(&mut message_buf)?;
    Ok(message_buf)
}