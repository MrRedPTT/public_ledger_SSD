#[doc(inline)]
use std::fmt;
use std::fmt::Display;

use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pss::Pss;
use rsa::sha2::Sha256;

use crate::marco::auction::Auction;
use crate::marco::bid::Bid;
use crate::marco::transaction::Transaction;
use crate::marco::sha512hash::Sha512Hash;

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
    pub fn calc_hash(&mut self) -> String {
        if self.hash != "".to_string() {
            return self.hash.clone();
        }

        self.hash = self.data.to_hash();
        return self.hash.clone();
    }
    
    pub fn sign(&mut self, skey: RsaPrivateKey) -> String {
        if self.hash == "".to_string() {
            self.calc_hash();
        }

        let signature = skey.sign::<Pss>(Pss::new::<Sha256>(),
            &self.hash.clone().into_bytes()).unwrap();
        self.signature = String::from_utf8(signature).unwrap_or_default();

        return self.signature.clone();
    }

    pub fn verify(&self, pkey: RsaPublicKey) -> bool{
        if self.hash != "".to_string() { return false; }
        if self.hash != self.data.to_hash() {return false;}

        let res = pkey.verify::<Pss>(Pss::new::<Sha256>(),
            &self.hash.clone().into_bytes(), &self.signature.clone().into_bytes());

        match res {
            Ok(()) => true,
            Err(..) => false
        }
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
