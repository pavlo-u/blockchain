#[path = "../blockchain/block.rs"]
pub mod block;
#[path = "../blockchain/timestamp.rs"]
pub mod timestamp;

use block::{header::Header, transaction::Transaction, Block};
use serde::{Deserialize, Serialize};
use timestamp::current_timestamp;

use rand::Rng;
use std::clone::Clone;
use std::collections::LinkedList;
//
use std::vec::Vec; //vector of Blockchains

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: LinkedList<Block>,
}

impl Blockchain {
    //ctor
    pub fn new() -> Blockchain {
        //genesis block
        let mut blocks_list: LinkedList<Block> = LinkedList::new();
        let mut transaction_vec: Vec<Transaction> = Vec::new();
        transaction_vec.push(Transaction {
            from: String::from("Genesis"),
            to: String::from("You"),
            amount: 0,
        });
        let header = Header::new();
        let genesis: Block = Block {
            head: header.clone(),
            transaction: transaction_vec,
            hash: header.head_timestamp,
            previous_hash: String::from("Genesis has no previous hash"),
        }; //create genesis block
        blocks_list.push_back(genesis);
        Blockchain {
            blocks: blocks_list,
        }
    }
    pub fn mint(&mut self, mempool_part: Vec<Transaction>) {
        let mut mint_block = Block {
            //create block
            head: Header::new(),
            transaction: mempool_part,    //first transaction from the queue
            hash: String::new(),          //some hash
            previous_hash: String::new(), //hash of the last element
        };
        loop {
            let last_block_hash = self
                .blocks
                .back()
                .expect("Can`t get last block!")
                .hash
                .clone();
            mint_block.previous_hash = last_block_hash;
            mint_block.head.nonce = rand::thread_rng().gen(); //rand number
            let tmp_hash = mint_block.hash_func(); //hash with BigInt
            let v: Vec<&str> = tmp_hash.matches("1").collect(); //count of the 1
            if v.len() >= 6 {
                mint_block.hash = tmp_hash; //add hash to block
                mint_block.head.head_timestamp = current_timestamp(); //
                self.blocks.push_back(mint_block); //add block to Blockchain(LinkedList)
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn new_blockchain_test() {
        let some_blockch: Blockchain = Blockchain::new();
        assert_eq!(
            some_blockch
                .blocks
                .back()
                .expect("No blocks")
                .transaction
                .get(0)
                .expect("No transaction in block")
                .from,
            String::from("Genesis")
        );
        assert_eq!(some_blockch.blocks.len(), 1);
    }
    #[test]
    fn mint_test() {
        let mut some_blockch: Blockchain = Blockchain::new();
        let mempool = vec![Transaction {
            from: String::from("Mint"),
            to: String::from("Test"),
            amount: 0,
        }];
        std::thread::sleep(Duration::new(1, 412));
        some_blockch.mint(mempool);
        assert_eq!(
            some_blockch
                .blocks
                .back()
                .expect("No blocks")
                .transaction
                .get(0)
                .expect("No transaction in block")
                .from,
            String::from("Mint")
        );
        assert_eq!(some_blockch.blocks.len(), 2);
        std::thread::sleep(Duration::new(1, 412));
        some_blockch.mint(vec![]);
        assert!(some_blockch
            .blocks
            .back()
            .expect("No blocks")
            .transaction
            .is_empty());
        assert_eq!(some_blockch.blocks.len(), 3);
        println!("{:?}", some_blockch);
    }
}
