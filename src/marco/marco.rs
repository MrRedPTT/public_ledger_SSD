#[doc(inline)]

use std::fmt;
use std::fmt::Display;

use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use crate::marco::auction::Auction;
use crate::marco::bid::Bid;
use crate::marco::sha512hash::Sha512Hash;
use crate::marco::transaction::Transaction;

///## MARCO
#[derive(Debug, Clone, PartialEq)]
pub struct Marco {
    pub(crate) hash: String,
    pub(crate) signature: String,
    pub data : Data
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Transaction(Transaction),
    CreateAuction(Auction),
    Bid(Bid)
}

impl Marco{
    pub fn hash(&mut self) -> String {
        if self.hash != "".to_string() {
            return self.hash;
        }

        self.hash = self.data.to_hash();
        return self.hash;
    }
    
    pub fn sign(&mut self, skey: RsaPrivateKey) -> String {
        if self.hash == "".to_string() {
            self.hash();
        }
        let enc_data = skey.sign(Pkcs1v15Encrypt, self.hash.into_bytes()).expect("failed to encrypt");
        assert_ne!(&data[..], &enc_data[..]);

        self.signature = String::from_utf8(signature).unwrap_or_default();

        return self.signature;
    }

    pub fn check_signature(&self, pkey: PKey<Public>) -> bool{
        if self.hash != "".to_string() { return false; }
        if self.hash != self.data.to_hash() {return false;}
        
        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
        verifier.update(&self.hash.into_bytes()).unwrap();
        return verifier.verify(&self.signature.into_bytes()).unwrap();
    }

    pub fn from_transaction(t: Transaction) -> Marco {
        Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            data: Data::Transaction(t)
        }
    }

    pub fn from_auction(a: Auction) -> Marco {
        Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            data: Data::CreateAuction(a)
        }
    }

    pub fn from_bid(b: Bid) -> Marco {
        Marco {
            hash: "".to_string(),
            signature: "".to_string(),
            data: Data::Bid(b)
        }
    }
    
    pub fn get_signature(&self) -> String { self.signature.clone()}
    pub fn get_hash(&self) -> String { self.hash.clone()}
}

impl Display for Marco { 
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            Data::Transaction(t) =>
                write!(f, "Data:{}, with hash {} and signature {}",
                       t.to_string(), self.hash, self.signature),
            Data::CreateAuction(a) => 
                write!(f, "Data:{}, with hash {} and signature {}", 
                       a.to_string(), self.hash, self.signature),
            Data::Bid(b) => 
                write!(f, "Data:{}, with hash {} and signature {}", 
                       b.to_string(), self.hash, self.signature),
        }
    }
}
impl Sha512Hash for Data {
    fn to_hash(&self) -> String {
        match self {
            Data::Transaction(t) => t.to_hash(),
            Data::CreateAuction(a) => a.to_hash(),
            Data::Bid(b) => b.to_hash(),
        }
    }
}

