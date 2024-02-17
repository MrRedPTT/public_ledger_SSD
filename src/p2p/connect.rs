use crate::p2p::peer::Peer;
pub fn connect() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "listen" => {
                let peer = Peer::new("127.0.0.1:7878".to_string());
                peer.start_listener();
            },
            "send" => {
                let peer = Peer::new("".to_string()); // No need for an address to send
                let peer_address = "127.0.0.1:7878";
                let message = "Hello, peer!";
                peer.connect_to_peer(peer_address, message);
            },
            _ => println!("Unknown command."),
        }
    } else {
        println!("Usage: program [listen | send]");
    }
}