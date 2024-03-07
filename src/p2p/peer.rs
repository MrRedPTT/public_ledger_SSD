use std::{io};
use std::time::Duration;
use async_std::net::{TcpListener, TcpStream};
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use crate::kademlia::node::Node;

pub struct Peer {
    node: Node,
    listener: TcpListener
}

impl Peer {

    pub async fn new(node: &Node) -> Result<Peer, io::Error> {
        let listener = TcpListener::bind(format!("{}:{}", node.ip, node.port)).await?;
        Ok(Peer {
            node: node.clone(),
            listener,
        })
    }

    pub async fn create_listener(self) -> std::io::Result<()>{
        let mut incoming = self.listener.incoming();

        while let Some(stream) = incoming.next().await {
            match stream {
                Ok(stream) => {
                    let _ = Self::handle_connection(stream).await;
                }
                Err(e) => eprintln!("Accept failed: {:?}", e),
            }
        }
        Ok(())
    }

    async fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error>{
        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).await?;

        // Security check to verify if the message was correctly obtained
        let message = Self::p2p_msg_to_string(buf);

        println!("Just received a connection from: {} with content: {:?} -> byte len: {:?}", stream.peer_addr().unwrap(), message, n);
        Ok(())
    }

    pub async fn create_sender(self, server: &Node) -> Result<(), io::Error>{
        // To test multiple connections I added a sleep (to be removed after the test)
        for _ in 0..10{
            tokio::time::sleep(Duration::from_millis(600)).await;

            let mut stream = TcpStream::connect(format!("{}:{}", server.ip, server.port)).await?;

            // Here to make the payload different for the 2 nodes while testing
            // When deploying remove the for (leaving the code that is inside)
            // And removing the following if else statement
            let mut payload: &[u8] = b"";
            if self.node.port == 9988 {
                payload = b"Hey There Brother";
            } else {
                payload = b"Hello World";
            }
            stream.write_all(payload).await?;
        }
        Ok(())
    }

    fn p2p_msg_to_string(msg: Vec<u8>) -> String {
        if msg.len() == 0 {
            return "".to_string();
        }
        let mut buf: Vec<u8> = vec![];
        for i in 0..msg.len() {
            if msg[i] != 0 {
                buf.push(msg[i]);
            } else {
                break;
            }
        }
        // The only element was a null byte
        if buf.len() == 0 {
            return "".to_string();
        }

        std::str::from_utf8(&buf).unwrap().to_string()

    }

}