#[doc(inline)]
use std::borrow::BorrowMut;
//use std::thread;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};

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
    ///vector of all your
    pub your_bids: HashMap<String,f64>,
    pub open: HashMap<String,Marco>,
    ///max bid for auction with hash
    pub all_bids: HashMap<String,f64>,
}


impl Auction {
    fn update_open_auctions(&mut self){
        //search bc for auctions
        let list = self.client.blockchain.lock().unwrap()
            .marco_set.clone();

        //println!("{:?}",list);
        let _:Vec<_> = list.into_iter()
            .map(|(key, value)| (
                match &value.data {
                    Data::CreateAuction(_) => {
                        self.open.insert(key,value.clone());
                    },
                    Data::Bid(b) => {
                        //update all_bids
                        let res = self.all_bids.get(&b.auction_id);
                        match res {
                            None => {
                                self.all_bids.insert(b.auction_id.clone() ,b.amount);
                            },
                            Some(bid) => {
                                if bid < &b.amount {
                                    self.all_bids.insert(b.auction_id.clone() ,b.amount);

                                }
                            }
                        };

                        //update my_bids
                        let res = self.your_bids.get(&b.auction_id);
                        match res {
                            None => {
                                    //println!("NO Bid for auction you subscribe");
                            },
                            Some(bid) => {
                                if bid < &b.amount {
                                    println!("New Bid for auction you subscribe");
                                    let mut entries: Vec<_> = self.open.iter().collect();
                                    entries.sort_by(|a, b| a.0.cmp(b.0));
                                    let mut i = 0;
                                    for (k, _) in entries {
                                        if k == &b.auction_id {
                                            println!("Auction id: {}", i);
                                            break;
                                        }
                                        i+=1;
                                    }

                                    self.your_bids.insert(b.auction_id.clone() ,b.amount);
                                    println!("Bid of {} by someone", b.amount);
                                }
                                else {
                                    //println!("Lower Bid for auction you subscribe");
                                }
                            }
                        };
                        println!("");
                        
                    },
                    _ => {}
                }
            )).collect();
    }

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
            self.update_open_auctions();
            println!("Choose an action:");
            println!("1. Open New Auction");
            println!("2. Place Bid");
            println!("3. Show Auctions");
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
            your_bids: HashMap::new(),
            all_bids: HashMap::new(),
        }
    }

    // Acquire information from the stdin to populate the Marco object

    // This will broadcast the Marco through the network
    // self.client.send_marco(marco).await;
    pub fn open_auction(&mut self) {
        let mut entries: Vec<_> = self.open.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (_, m) in entries {
            match &m.data {
                Data::CreateAuction(a)=> {
                    if a.seller_id == self.id {
                        println!("You can only have 1 open auction at the same time\n");
                        return;
                    }
                }
                _ => {}
            }
        }

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
        entries.sort_by(|a, b| a.0.cmp(b.0));
        if  auction_id >= entries.len() {
            return ;
        }
        let auction :& Marco = entries[auction_id].1;

        match &auction.data {
            Data::CreateAuction(a) => {

                let mut m = Marco::from_bid(Bid::new(auction.get_hash(),self.id.clone(), a.seller_id.clone(), value.clone() ));
                self.your_bids.insert(auction.get_hash(), value);
                self.add_and_broadcast(m.borrow_mut());
            }
            _ => {}
        }
    }

    pub fn check_winner(&self) {
        //buscar hash de auction
        let mut entries: Vec<_> = self.open.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (_, value) in entries {
            match &value.data {
                Data::CreateAuction(a) => {
                    if a.seller_id == self.id {
                        println!("You can only have 1 open auction at the same time\n");

                        return;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn search_auctions(&mut self) {
        self.update_open_auctions();

        //print auction map
        println!("Printing List of Auction");
        let mut entries: Vec<_> = self.open.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut i = 0;
        for (_, value) in entries {
            let amount: f64;
            match &value.data {
                Data::CreateAuction(a) => {
                    amount = a.amount;
                    if a.seller_id == self.id {
                        continue;
                    }
                }
                _ => {continue;}
            }

            println!("Auction {}: Auctioning {} {}ubiously {}nsecure {}oin {}eeper(s)",i, amount,
                "D".bold().bright_red(),
                "I".bold().bright_yellow(),
                "C".bold().bright_blue(),
                "K".bold().bright_magenta());
            
            let res = self.all_bids.get(&value.to_hash());
            match res  {
                None => println!("\t Currently, this auction does not have a bid"),
                Some(b) => println!("\thighest bid is {} euros", b),
            }
            
            i+=1;
        }
    }

    pub fn close_auction(&self) {
        loop {
            let x = self.get_user_input("Do you actually want to finish your auction early?\n");
            let result = x.trim().parse::<bool>();

            match result {
                Ok(_) => {
                    break;
                }
                Err(e) => println!("A positive integer is needed: {}", e),
            }
        }
        self.check_winner();
    }
}
