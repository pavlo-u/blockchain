
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Header {
    pub head_timestamp: String,
    pub nonce: usize,
}
