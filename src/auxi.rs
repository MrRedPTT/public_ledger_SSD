use pem::{encode, Pem};
use rand::Rng;
use rsa::{pkcs1::DecodeRsaPrivateKey,pkcs1::DecodeRsaPublicKey, RsaPrivateKey, RsaPublicKey};
use sha3::{Digest, Sha3_256};
use tokio::net::TcpListener;
use x509_certificate::X509Certificate;

use proto::{Auction as ProtoAuction, Bid as ProtoBid, Data as ProtoData, Marco as ProtoMarco, Transaction as ProtoTransaction};
use proto::data::DataType as ProtoDataType;

use crate::kademlia::node::ID_LEN;
// Auxiliary functions
#[doc(inline)]
use crate::kademlia::node::Identifier;
use crate::marco::auction::Auction;
use crate::marco::bid::Bid;
use crate::marco::marco::{Data, Marco};
use crate::marco::transaction::Transaction;
use crate::proto;
use crate::proto::DstAddress;
use crate::proto::SrcAddress;

/// Converts a node identifier (Vec<u8>) into a string, after hashing
pub fn convert_node_id_to_string (node_id: &Identifier) -> String{
    let mut hasher = Sha3_256::new();
    hasher.update(node_id.0);
    let hashed_node_id= hasher.finalize();
    let string = hashed_node_id.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    return string;
}

/// Converts an Identifier (`Vec<u8>`) into a String
pub fn vec_u8_to_string (v: Identifier) -> String {
    let mut string_result: String = "".parse().unwrap();
    for x in &v.0 {
        string_result += &*x.to_string();
    }
    return string_result;
}

// Applies XOR to each element of the Vector
// and returns the XORed elements inside another Vector
/// Calculates the distance between two Identifiers using xor
pub fn xor_distance(id1: &Identifier, id2: &Identifier) -> usize {
    let mut res: [u8; ID_LEN] = [0; ID_LEN];
    for i in 0..ID_LEN {
        res[i] = id1.0[i] ^ id2.0[i];
    }

    let leading = get_leading(res) as usize;
    if leading == 0 {
        return 1;
    }
    return leading;
}

// Return the number of leading zeros (aka number of bits different between the two nodes)
/// Gets the number of leading zeros, i.e., the number of consecutive zeros on the left
fn get_leading(v: [u8; ID_LEN]) -> u32 {
    let mut count = 0;
    for i in v {
        if i == 0 {
            count += 1;
        } else {
            return count;
        }
    }
    return count;
}

pub fn gen_id (str: String) -> Identifier {
    let mut hasher = Sha3_256::new();
    hasher.update(str.as_bytes());

    // we know that the hash output is going to be 256 bits = 32 bytes
    let result = hasher.finalize();

    let mut hash = [0; ID_LEN];
    let mut bin_str = "".to_string();
    for b in result {
        bin_str += &format!("{:0>8b}", b); // Force the representation to pad with 0's in case there aren't 8 bits
    }

    let mut i = 0;
    assert_eq!(bin_str.len(), ID_LEN); // assert that we indeed have ID_LEN bits
    for b in bin_str.chars() {
        hash[i] = b.to_digit(2).unwrap() as u8;
        i += 1;
    }
    Identifier::new(hash)
}

pub fn gen_address_src(id: Identifier,ip: String, port: u32) -> Option<SrcAddress> {
    Some(proto::SrcAddress {
        id: id.0.to_vec(),
        ip,
        port
    })
}

pub fn gen_address_dst(ip: String, port: u32) -> Option<DstAddress> {
    Some(proto::DstAddress {
        ip,
        port
    })
}

pub fn return_option<T>(arg: T) -> Option<T> {
    Some(arg)
}

pub async fn get_port() -> u16 {
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(1025..=65535);
    loop {
        match TcpListener::bind(("127.0.0.1", random_number)).await {
            Ok(_) => {
                // Binding succeeded, port is not in use
                return random_number;
            }
            Err(_) => {
                // Binding failed, port is already in use
            }
        }
    }
}

pub fn get_public_key(cert_pem: String) -> RsaPublicKey {
    let cert = &X509Certificate::from_pem(cert_pem.as_bytes()).unwrap();

    // Pick public key
    let public_key = &X509Certificate::public_key_data(&cert);

    // Export public key as PEM encoded public key in PKCS#1 format
    let pem = Pem::new(String::from("RSA PUBLIC KEY"), public_key.to_vec());
    let public_pkcs1_pem = encode(&pem);
    let public_key = RsaPublicKey::from_pkcs1_pem(public_pkcs1_pem.as_str()).expect("Failed to Retrieve public key");
    return public_key;
}
pub fn get_private_key(private_key: String) -> RsaPrivateKey {
    // Parse the PEM-encoded private key string
    //let pem_key = pem::parse(private_key_str.as_str()).expect("Failed to parse private key");

    // Parse the DER-encoded key into components
    let rsa_priv = RsaPrivateKey::from_pkcs1_pem(private_key.as_str()).expect("Failed to generate private key");
    rsa_priv
}

pub fn transform_proto_to_marco(proto_marco: &ProtoMarco) -> Marco {
    let data = match proto_marco.data.as_ref().expect("Data is missing").data_type.as_ref().expect("Data type is missing") {
        ProtoDataType::Transaction(t) => Data::Transaction(Transaction {
            from: t.from.clone(),
            to: t.to.clone(),
            amount_in: t.amount_in,
            amount_out: t.amount_out,
            miner_fee: t.miner_fee,
        }),
        ProtoDataType::CreateAuction(a) => Data::CreateAuction(Auction {
            auction_id: a.auction_id,
            seller_id: a.seller_id.clone(),
            amount: a.amount,
        }),
        ProtoDataType::Bid(b) => Data::Bid(Bid {
            auction_id: b.auction_id,
            buyer_id: b.buyer_id.clone(),
            seller_id: b.seller_id.clone(),
            amount: b.amount,
        }),
    };

    Marco {
        hash: proto_marco.hash.clone(),
        signature: proto_marco.signature.clone(),
        data,
        timestamp: timestamp_to_system_time(Some(&proto_marco.timestamp.clone().unwrap())).unwrap()
    }
}

pub fn transform_marco_to_proto(marco: &Marco) -> ProtoMarco {
    let data = match &marco.data {
        Data::Transaction(t) => ProtoData {
            data_type: Some(ProtoDataType::Transaction(ProtoTransaction {
                from: t.from.clone(),
                to: t.to.clone(),
                amount_in: t.amount_in,
                amount_out: t.amount_out,
                miner_fee: t.miner_fee,
            })),
        },
        Data::CreateAuction(a) => ProtoData {
            data_type: Some(ProtoDataType::CreateAuction(ProtoAuction {
                auction_id: a.auction_id,
                seller_id: a.seller_id.clone(),
                amount: a.amount,
            })),
        },
        Data::Bid(b) => ProtoData {
            data_type: Some(ProtoDataType::Bid(ProtoBid {
                auction_id: b.auction_id,
                buyer_id: b.buyer_id.clone(),
                seller_id: b.seller_id.clone(),
                amount: b.amount,
            })),
        },
    };

    proto::Marco {
        hash: marco.hash.clone(),
        signature: marco.signature.clone(),
        data: Some(data),
        timestamp: system_time_to_timestamp(Some(marco.timestamp))
    }
}

// Function to convert Option<SystemTime> to Option<protobuf Timestamp>
fn system_time_to_timestamp(system_time: Option<std::time::SystemTime>) -> Option<prost_types::Timestamp> {
    system_time.map(|st| {
        let duration_since_epoch = st.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
        prost_types::Timestamp {
            seconds: duration_since_epoch.as_secs() as i64,
            nanos: duration_since_epoch.subsec_nanos() as i32,
        }
    })
}

// Function to convert Option<protobuf Timestamp> to Option<SystemTime>
fn timestamp_to_system_time(timestamp: Option<&prost_types::Timestamp>) -> Option<std::time::SystemTime> {
    timestamp.map(|ts| std::time::UNIX_EPOCH + std::time::Duration::new(ts.seconds as u64, ts.nanos as u32))
}
