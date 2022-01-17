use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Header {
    pub head_timestamp: String,
    pub nonce: usize,
}
