use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]

pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}
impl Clone for Transaction {
    fn clone(&self) -> Transaction {
        Transaction {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount.clone(),
        }
    }
}
