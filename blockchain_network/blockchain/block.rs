use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256}; //hash
#[path = "../blockchain/header.rs"]
pub mod header;
#[path = "../blockchain/transaction.rs"]
pub mod transaction;

use header::Header;
use transaction::Transaction;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub head: Header,
    pub transaction: Vec<Transaction>,
    pub hash: String,
    pub previous_hash: String,
}
impl Block {
    //hash function
    pub fn hash_func(&self) -> String {
        let mut trnsctn =
            self.head.head_timestamp.clone() + &self.head.nonce.to_string() + &self.previous_hash;
        for transaction in &self.transaction {
            trnsctn =
                trnsctn + &transaction.from + &transaction.to + &transaction.amount.to_string()
        }
        let mut hasher = Sha256::new();
        hasher.update(trnsctn);
        format!("{:x}", hasher.finalize())
    }
}
