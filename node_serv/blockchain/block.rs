use sha2::{Digest, Sha256}; //hash
#[path = "../blockchain/header.rs"]
pub mod header;
#[path = "../blockchain/transaction.rs"]
pub mod transaction;

use header::Header;
use transaction::Transaction;

#[derive(PartialEq, Debug)]
pub struct Block {
    pub head: Header,
    pub transaction: Transaction,
    pub hash: String,
    pub previous_hash: String,
}
impl Clone for Block {
    fn clone(&self) -> Block {
        Block {
            head: self.head.clone(),
            transaction: self.transaction.clone(),
            hash: self.hash.clone(),
            previous_hash: self.previous_hash.clone(),
        }
    }
}
impl Block {
    //hash function
    pub fn hash_func(&mut self) -> String {
        //concat
        let trnsctn = self.head.head_timestamp.clone()
            + &self.head.nonce.to_string()
            + &self.transaction.from
            + &self.transaction.to
            + &self.transaction.amount.to_string()
            + &self.previous_hash
            + &self.head.nonce.to_string();
        let mut hasher = Sha256::new();
        hasher.update(trnsctn);
        format!("{:x}", hasher.finalize())
    }
}
