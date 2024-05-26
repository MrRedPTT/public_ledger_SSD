use std::borrow::BorrowMut;
//use std::thread;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};

#[doc(inline)]
use colored::Colorize;
//use std::time::Duration;
use rsa::{pkcs1v15::SigningKey, pkcs8::DecodePrivateKey, RsaPublicKey};
use rsa::sha2::Sha256;

use crate::auxi;
use crate::kademlia::node::Node;
use crate::marco::auction::Auction as MarcoAuction;
use crate::marco::bid::Bid;
use crate::marco::marco::{Data, Marco};
use crate::p2p::peer::Peer;

pub struct Auction {
    pub client: Peer,
    pub id:String,
    pub pkey:RsaPublicKey,
    pub skey: SigningKey<Sha256>,
    pub open: HashMap<String,MarcoAuction>,
}


impl Auction {
    fn get_user_input(&self, prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input
    }

    fn print_bc(&self){
        let list = self.client.blockchain.lock().unwrap().chain.clone();
        for i in list {
            println!("Block: id: {{{}}} hash:{}",i.index.clone(), i.hash.clone());
        }
        let list = self.client.blockchain.lock().unwrap().heads.get_main().clone();
        for i in list {
            println!("Block{{Head}}: id: {{{}}} hash:{}",i.index.clone(), i.hash.clone());
        }
    }

    fn add_and_broadcast(&self, m: &mut Marco){
        m.calc_hash();
        m.sign(self.skey.clone());
        let (res,ob) = self.client.blockchain.lock().unwrap().add_marco(m.clone(),self.pkey.clone());
        if !res {
            println!("There was an issue with the generated auction");
            return;
        }
        match ob {
            None => {
                let local_client = self.client.clone();
                let m_clone = m.clone();
                tokio::spawn(async move {
                    //open auctin with value
                    local_client.send_marco(m_clone).await
                });
            },
            Some(b) => {
                let local_client = self.client.clone();
                tokio::spawn(async move {
                    //open auctin with value
                    local_client.send_block(b).await
                });
            },
        }
    }

    pub async fn main(&mut self) {
        loop {
            println!("Choose an action:");
            println!("1. Open Auction");
            println!("2. Place Bid");
            println!("3. Show Auction");
            println!("4. Print The BlockChain");
            println!("5. Exit");

            let choice = self.get_user_input("Enter your choice: ");
            match choice.trim() {
                "1" => self.open_auction(),
                "2" => self.place_bid(),
                "3" => self.search_auctions(),
                "4" => self.print_bc(),
                "5" => {
                    println!("Exiting...");
                    break;
                },
                _ => println!("Invalid choice, please try again."),
            }
        }
    }

    pub async fn new() -> Self {
        let node = Node::new("127.0.0.1".to_string(), auxi::get_port().await as u32);
        let (client, server) = Peer::new(&node.unwrap(), false);
        let _ = server.init_server().await;
        client.boot().await;

        let id = client.id.0.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");

        let mut slash = "/";
        if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "windows" {
            slash = "\\";
        }
        let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR")]);
        let pem = std::fs::read_to_string(data_dir.join(format!("cert{slash}server.crt"))).expect("Failed to read server.crt");
        let pkey = auxi::get_public_key(pem);
        //let mut rng = rand::thread_rng();
        //let bits = 2048;
        //let skey = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let skey = SigningKey::read_pkcs8_pem_file(data_dir.join(format!("cert{slash}server.key"))).expect("Failed to read server.key");

        Auction {
            client,
            id,
            pkey,
            skey,
            open: HashMap::new(),
        }
    }

    // Acquire information from the stdin to populate the Marco object

    // This will broadcast the Marco through the network
    // self.client.send_marco(marco).await;
    pub fn open_auction(&self) {
        let value :f64;
        loop {
            let x = self.get_user_input("How many coins do you want to auction?\n");
            let result = x.trim().parse::<f64>();

            match result {
                Ok(number) => {
                    value = number;
                    break;
                }
                Err(e) => println!("A float is needed: {}", e),
            }
        }
        let mut m = Marco::from_auction(MarcoAuction::new(self.id.clone(), value));
        m.calc_hash();
        m.sign(self.skey.clone());
        self.add_and_broadcast(m.borrow_mut());
    }

    pub fn place_bid(&mut self) {
        self.search_auctions();
        let mut entries: Vec<_> = self.open.iter().collect();
        if entries.len() == 0 {
            println!("No Auctions Found!");
            return;
        }

        let mut auction_id :usize;
        loop {
            let x = self.get_user_input("What auction do you want to bid in?\n");
            let result = x.trim().parse::<usize>();

            match result {
                Ok(number) => {
                    auction_id = number;
                    if  auction_id < entries.len() {
                        break;
                    } else {
                        println!("A positive integer, between 0 and {}, is needed",entries.len());
                    }
                }
                Err(e) => println!("A positive integer is needed: {}", e),
            }
        }
        let value :f64;
        loop {
            let x = self.get_user_input("How many euros do you want to bid?\n");
            let result = x.trim().parse::<f64>();

            match result {
                Ok(number) => {
                    value = number;
                    break;
                }
                Err(e) => println!("A float is needed: {}", e),
            }
        }

        //ir buscar auction
        let mut auction:&MarcoAuction = &MarcoAuction::new("".to_string(),0.0);
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut i :usize= 0;
        if  auction_id >= entries.len() {
            return ;
        }
        for (_,value) in entries.iter().clone() {
            if i != auction_id{
                break;
            }
            auction = value;
            i+=1;
        }




        let mut m = Marco::from_bid(Bid::new(self.id.clone(), auction.seller_id.clone(), value ));
        m.calc_hash();
        //m.sign(self.skey.clone());
        self.add_and_broadcast(m.borrow_mut());

        // Acquire information from the stdin to populate the Marco object

        // Once again, use the following call to broadcast the Marco
        // self.client.send_marco(marco).await;
    }

    pub fn winner(&self) {
        todo!();
        // Acquire information from the stdin to populate the Marco object

        // Once again, use the following call to broadcast the Marco
        // self.client.send_marco(marco).await;
    }

    pub fn search_auctions(&mut self) {
        //search bc for auctions
        let list = self.client.blockchain.lock().unwrap()
            .marco_set.clone();

        //println!("{:?}",list);
        let _:Vec<_> = list.into_iter()
            .map(|(key, value)| (
                match value.data {
                    Data::CreateAuction(a) => {
                        self.open.insert(key,a);
                    },
                    _ => {}
                }
            )).collect();

        //print auction map
        println!("Printing List of Auction");
        let mut entries: Vec<_> = self.open.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut i = 0;
        for (_, value) in entries {
            println!("Auction {}: Auctioning {} {}ubiously {}nsecure {}oin {}eeper(s)",i, value.amount,
                "D".bold().bright_red(),
                "I".bold().bright_yellow(),
                "C".bold().bright_blue(),
                "K".bold().bright_magenta());
            i+=1;
        }

    }

}
