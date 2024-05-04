use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main () -> Result<(), Box<dyn std::error::Error>> {
    let _ = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .out_dir("proto/")
        .compile(&["proto/rpc_packet.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/rpc_packet.proto")?;

    cert_gen();

    Ok(())
}

fn cert_gen() {

    // Openssl comes with git, so to avoid installing 3rd party software
    // if the user has openssl from git, use it
    let file_path = "C:\\Program Files\\Git\\usr\\bin\\openssl.exe";
    let mut openssl = "openssl".to_string();
    if fs::metadata(&file_path).is_ok() {
        openssl = file_path.to_string();
    }
    // Check if certificates already exist
    if !Path::new("cert\\server.key").exists() || !Path::new("cert\\server.crt").exists() {
        println!("Certificates not found, generating new ones...");

        // Create cert directory if it doesn't exist
        std::fs::create_dir_all("cert").expect("Failed to create cert directory");

        // Generate private key
        let _ = Command::new(openssl.clone())
            .args(&["genpkey", "-algorithm", "RSA", "-out", "cert\\server.key"])
            .status();

        // Generate CSR
        let _ = Command::new(openssl.clone())
            .args(&["req", "-new", "-key", "cert\\server.key", "-out", "cert\\server.csr", "-subj", "\"/C=US/ST=California/L=Los Angeles/O=Example Company/CN=example.com\""])
            .status();

        // Generate self-signed certificate
        let _ = Command::new(openssl.clone())
            .args(&["x509", "-req", "-days", "365", "-in", "cert\\server.csr", "-signkey", "cert\\server.key", "-out", "cert\\server.crt"])
            .status();
    } else {
        println!("Certificates already exist, skipping generation.");
    }
}