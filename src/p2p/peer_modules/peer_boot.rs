use std::{env, fs, io};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::exit;

use crate::kademlia::node::Node;
use crate::p2p::peer::Peer;

impl Peer {

    pub async fn boot(&self) {
        if self.bootstrap {
            return;
        }

        let path;
        if let Ok(mut exe_path) = std::env::current_dir() {
            exe_path.push("src");
            exe_path.push("main");
            if let Some(dir) = exe_path.parent() {
                if env::var("OS_CONF").unwrap_or_else(|_| "linux".to_string()) == "windows" {
                    path = dir.display().to_string() + "\\bootstrap.txt";
                } else {
                    path = dir.display().to_string() + "/bootstrap.txt";
                }
                if let Ok(exists) = Self::file_exists(path.as_str()) {
                    if exists {
                        if let Ok(mut bootstrap_nodes) = Self::read_ips_from_file(path.as_str()) {
                            println!("List of Bootstrap nodes stored:");
                            for i in &bootstrap_nodes {
                                println!("{}", i);
                            }
                            let skip_list = env::var("DEFAULT_BOOTSTRAP").unwrap_or_else(|_| "y".to_string());
                            if skip_list != "y" {
                                println!("Do you wish to rewrite the list? (y/n)");
                                let mut input = String::new();
                                io::stdin().read_line(&mut input)
                                    .expect("Failed to read line");
                                if input.ends_with('\n') {
                                    input.pop();
                                    if input.ends_with('\r') {
                                        input.pop();
                                    }
                                }
                                if input.to_ascii_lowercase() == "y" {
                                    bootstrap_nodes = Self::read_ips_from_user();
                                    let _ = Self::write_file(path.as_str(), bootstrap_nodes.clone());
                                }
                            }
                            for i in &bootstrap_nodes {
                                self.kademlia.lock().unwrap().add_node(&Node::new(i.to_string(), 8635).unwrap());
                            }
                        }
                    } else {
                        println!("No bootstrap ip list was found!");
                        let bootstrap_nodes = Self::read_ips_from_user();
                        println!("DEBUG PEER_BOOT => Path: {}", path.as_str());
                        let _ = Self::create_file(path.as_str(), bootstrap_nodes.clone());
                        for i in bootstrap_nodes {
                            self.kademlia.lock().unwrap().add_node(&Node::new(i, 8635).unwrap());
                        }
                    }
                } else {
                    println!("DEBUG BOOT => Error oppening file");
                }
            } else {
                println!("Unable to determine main file path");
                exit(1);
            }
        } else {
            println!("Unable to get current executable path");
            exit(1);
        }

        let _ = self.find_node(self.node.id.clone()).await;

    }

    // Function to check if the file exists
    fn file_exists(file_path: &str) -> std::io::Result<bool> {
        println!("DEBUG PEER_BOOT::FILE_EXISTS => Was queried for file: {file_path}");
        let metadata = fs::metadata(file_path);
        match metadata {
            Ok(_) => Ok(true), // File exists
            Err(_) => Ok(false), // File does not exist
        }
    }

    // Function to create the file
    fn create_file(file_path: &str, ips: Vec<String>) -> std::io::Result<()> {
        let mut file = fs::File::create(file_path)?;
        for i in ips {
            file.write_all(i.as_bytes())?;
        }
        Ok(())
    }

    fn write_file(file_path: &str, lines: Vec<String>) -> io::Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        for line in lines {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        writer.flush()?;

        Ok(())
    }

    fn read_ips_from_file(file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let mut ips = Vec::new();

        for line in reader.lines() {
            if let Ok(ip) = line?.parse::<std::net::IpAddr>() {
                ips.push(ip.to_string());
            }
        }

        Ok(ips)
    }

    fn read_ips_from_user() -> Vec<String>{
        println!("Please enter IP addresses (IPv4 or IPv6) one per line. Enter a blank line to stop:");
        let mut ips: Vec<String> = Vec::new();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .expect("Failed to read line");

            let trimmed_input = input.trim();

            // If the input is empty (only newline), break the loop
            if trimmed_input.is_empty() {
                break;
            }

            // Attempt to parse the input into an IpAddr
            if let Ok(ip_addr) = trimmed_input.parse::<std::net::IpAddr>() {
                ips.push(ip_addr.to_string());
            } else {
                println!("Invalid IP address: {}", trimmed_input);
            }
        }
        return ips;
    }
}