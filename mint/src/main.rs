use num_bigint::{RandBigInt, BigInt}; //
use sha2::{Sha256, Digest}; //hash
use std::collections::LinkedList; //blockchain
use std::collections::VecDeque; //queue
use std::time::{SystemTime, UNIX_EPOCH};//timestamp
/*////////////////////////////////
Blockchain emulation
*/////////////////////////////////
#[derive(Debug)]     
struct Transaction {
    from: String, 
    to: String, 
    amount: u64
}
/*
impl Copy for Transaction {
    fn copy(&self) 
}*/
impl Clone for Transaction {
    fn clone(&self) -> Transaction {
        Transaction {
            from: self.from.clone(),
            to: self.to.clone(),
            amount: self.amount.clone(),
        }
    }
}
#[derive(Debug)]
struct Header {
    head_timestamp: String,
    nonce: BigInt
}

#[derive(Debug)]
struct Block {
    head: Header,
    transaction: Transaction,
    hash: String,
    previous_hash: String
}

impl Block {
    //hash function
    /*fn hash_func(&self, trans: Transaction) -> String { //concat
        let mut trnsctn = trans.from.to_string() + &trans.to; //from + to
        trnsctn = trnsctn + &trans.amount.to_string();//from+to + amount
        trnsctn = trnsctn + &self.hash;//from+to+amount + previous hash
        let mut hasher = Sha256::new();
        hasher.update(trnsctn);
        format!("{:x}", hasher.finalize())
    }*/
    fn hash_func(&mut self) -> String { //concat
        let mut trnsctn = self.transaction.from.clone() + &self.transaction.to; //from + to
        trnsctn = trnsctn + &self.transaction.amount.to_string();//from+to + amount
        trnsctn = trnsctn + &self.previous_hash;//from+to+amount + previous hash
        trnsctn = trnsctn + &self.head.nonce.to_string();
        let mut hasher = Sha256::new();
        hasher.update(trnsctn);
        format!("{:x}", hasher.finalize())
    }
}

pub fn current_timestamp() -> String {
    let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
       since_the_epoch.as_millis().to_string()
}

#[derive(Debug)]
struct Blockchain{
    blocks: LinkedList<Block>, 
    transactions_queue: VecDeque<Transaction> //очередь на обработку
}
impl Blockchain {
    //ctor
    fn new() -> Blockchain {
        //create timestamp
        //let timestamp = current_timestamp();
        //genesis block
        let transac_queue = VecDeque::new(); //create queue
        let mut bloks_list: LinkedList<Block> = LinkedList::new();//create List
        let genesis = Block { 
            head: Header {
            head_timestamp: current_timestamp(),
            nonce: rand::thread_rng().gen_bigint(1000) //random number
            },
            transaction: Transaction {
            from: "Genesis".to_string(), 
            to: "".to_string(), 
            amount: 0,
            }, 
            hash: current_timestamp(),
            previous_hash: "".to_string()
        }; //create genesis block
        bloks_list.push_back(genesis);
        Blockchain { 
            blocks: bloks_list, 
            transactions_queue: transac_queue,
        }
    }
    fn new_transaction(&mut self, from: String, to: String, amount: u64){
        self.transactions_queue.push_back(Transaction {
            from,
            to,
            amount
        }); 
    }
}
impl Blockchain {
    fn mint(&mut self) {
        if self.transactions_queue.is_empty() == false //check queue
        {
            let last_block = self.blocks.back().unwrap();//last element of the list
            let mut mint_block = Block {  //create block
                head: Header { 
                    head_timestamp: String::from(""), //some timestamp
                    nonce: rand::thread_rng().gen_bigint(1000) //some number
                },
                transaction: self.transactions_queue.pop_front().unwrap().clone(), //first transaction from the queue
                hash: current_timestamp(),//some hash
                previous_hash: last_block.hash.to_string() //hash of the last element
            };            
            loop {
                mint_block.head.nonce = rand::thread_rng().gen_bigint(1000);//rand BigInt number
                let tmp_hash = mint_block.hash_func(); //hash with BigInt 
                let v: Vec<&str> = tmp_hash.matches("1").collect();//count of the 1
                //println!("\nin loop\n{}\n", tmp_hash); 
                if v.len() >= 6 { 
                    mint_block.hash = tmp_hash; //add hash to block
                    mint_block.head.head_timestamp = current_timestamp(); //
                    self.blocks.push_back(mint_block); //add block to Blockchain(LinkedList)
                    break;
                }
            }
        } else {
            println!("Queue is empty!");
        } 
    }
}

fn main() {
    let mut blockch: Blockchain = Blockchain::new(); //create Blockchain
    println!("Before mint and transaction {:?}\n", blockch);
    blockch.new_transaction("Sender".to_string(), "Recipient".to_string(), 27);
    println!("Before mint and after transaction {:?}\n", blockch);
    blockch.mint();
    println!("\nAfter mint and transaction\n{:?}\n", blockch);
}