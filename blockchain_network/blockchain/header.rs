use serde::{Deserialize, Serialize};
#[path = "../blockchain/timestamp.rs"]
pub mod timestamp;
use rand::Rng;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Header {
    pub head_timestamp: String,
    pub nonce: usize,
}
impl Header {
    pub fn new() -> Header {
        Header {
            head_timestamp: timestamp::current_timestamp(),
            nonce: rand::thread_rng().gen(), //random number
        }
    }
}
