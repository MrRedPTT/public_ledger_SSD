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
                let peer = Peer::new("".to_string()); // This address is not used for sending
                let peer_address = "127.0.0.1:7878";
                let messages = vec!["Hello, peer!", "How are you?", "BYE"];
                peer.connect_to_peer(peer_address, messages);
            },
            _ => println!("Unknown command."),
        }
    } else {
        println!("Usage: program [listen | send]");
    }
}