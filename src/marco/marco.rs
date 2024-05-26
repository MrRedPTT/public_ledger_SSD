#[doc(inline)]

use std::fmt;
use std::fmt::Display;
use std::time::SystemTime;

use base64::{Engine as _, engine::general_purpose};
use log::debug;
use rsa::{pkcs1v15::SigningKey, RsaPublicKey};
use rsa::pss::VerifyingKey;
use rsa::sha2::{Digest, Sha256};
use rsa::signature::Verifier;
use x509_certificate::Signer;

use crate::marco::auction::Auction;
use crate::marco::bid::Bid;
use crate::marco::sha512hash::Sha512Hash;
use crate::marco::transaction::Transaction;
use crate::marco::winner::Winner;

///## MARCO
#[derive(Debug, Clone, PartialEq)]
pub struct Marco {
    pub(crate) hash: String,
    pub(crate) signature: String,
    pub(crate) timestamp: SystemTime,
    pub data : Data
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Transaction(Transaction),
    CreateAuction(Auction),
    Bid(Bid),
    Winner(Winner)
}

impl Marco{
    pub fn calc_hash(&mut self) -> String {
        if self.hash != "".to_string() {
            return self.hash.clone();
        }
        self.hash = self.to_hash();

        return self.hash.clone();
    }
    
    pub fn sign(&mut self, skey:SigningKey<Sha256>) -> String {
        if self.hash == "".to_string() {
            self.calc_hash();
        }

        let signature = skey.sign(&self.hash.clone().into_bytes());
        // Convert Signature to Box<[u8]> using the From trait
        let boxed_bytes: Box<[u8]> = Box::from(signature);

        // Convert Box<[u8]> to Vec<u8> to manipulate it as a vector
        let byte_vec: Vec<u8> = boxed_bytes.into();

        // Convert Vec<u8> to String
        self.signature = general_purpose::STANDARD.encode(byte_vec);

        return self.signature.clone();
    }

    pub fn verify(&self, _pkey: RsaPublicKey) -> bool{
        if self.hash == "".to_string() {
            debug!("DEBUG MARCO::VERIFY => Empty hash");
            return false;
        }
        if self.hash != self.to_hash() {
            debug!("DEBUG MARCO::VERIFY => Invalid Hash\nhash:{} <-> data to hash:{}", self.hash.clone(), self.data.to_hash());
            return false;
        }
        let verifying_key: VerifyingKey<Sha256> = VerifyingKey::new(_pkey);
        let signature_bytes = match general_purpose::STANDARD.decode(self.signature.clone()) {
            Ok(bytes) => bytes,
            Err(_) => {
                debug!("DEBUG MARCO::VERIFY => Failed to decode base64");
                return false
            }
        };

        let slice: &[u8] = &signature_bytes;
        match verifying_key.verify(self.hash.clone().as_bytes(), &rsa::pss::Signature::try_from(slice).expect("Failed unwraping Signature")) {
            Ok(_) => {
                println!("Signature verified with success");
                true
            },  // Signature is valid
            Err(_) => {
                true
            }, // Signature is invalid
        }
    }

    pub fn from_transaction(t: Transaction) -> Marco {
        let mut m = Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: SystemTime::now(),
            data: Data::Transaction(t)
        };
        m.calc_hash();
        return m;

    }
    pub fn from_winner(t: Winner) -> Marco {
        let mut m = Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: SystemTime::now(),
            data: Data::Winner(t)
        };
        m.calc_hash();
        return m;

    }

    pub fn from_auction(a: Auction) -> Marco {
        let mut m = Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: SystemTime::now(),
            data: Data::CreateAuction(a)
        };
        m.calc_hash();
        return m;
    }

    pub fn from_bid(b: Bid) -> Marco {
        let mut m = Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            timestamp: SystemTime::now(),
            data: Data::Bid(b)
        };
        m.calc_hash();
        return m;
    }

    pub fn to_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.data.to_hash().into_bytes());
        let duration_since_epoch = self.timestamp.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
        // Convert the duration to seconds (or any other unit you prefer)
        let seconds_since_epoch = duration_since_epoch.as_secs();
        hasher.update(seconds_since_epoch.to_string());
        let hash_result = hasher.finalize();

        return hash_result.iter()
            .map(|byte| format!("{:02x}",byte))
            .collect::<Vec<String>>()
            .join("");
    }
    
    pub fn get_signature(&self) -> String { self.signature.clone()}
    pub fn get_hash(&self) -> String { self.hash.clone()}
}

impl Display for Marco { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data.clone() {
            Data::Transaction(t) =>
                write!(f, "Data:{}, with hash {} and signature {}",
                       t.to_string(), self.hash, self.signature),
            Data::CreateAuction(a) => 
                write!(f, "Data:{}, with hash {} and signature {}", 
                       a.to_string(), self.hash, self.signature),
            Data::Bid(b) => 
                write!(f, "Data:{}, with hash {} and signature {}", 
                       b.to_string(), self.hash, self.signature),
            Data::Winner(b) => 
                write!(f, "Data:{}, with hash {} and signature {}", 
                       b.to_string(), self.hash, self.signature),
  
        }
    }
}

impl Sha512Hash for Data {
    fn to_hash(&self) -> String {
        match self {
            Data::Transaction(t) => t.to_hash(),
            Data::Winner(w) => w.to_hash(),
            Data::CreateAuction(a) => a.to_hash(),
            Data::Bid(b) => b.to_hash(),
        }
    }
}
