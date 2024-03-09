use std::env;
use std::path::PathBuf;

fn main () -> Result<(), Box<dyn std::error::Error>> {
    let _ = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .out_dir("proto/")
        .compile(&["proto/rpc_packet.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/rpc_packet.proto")?;

    Ok(())
}