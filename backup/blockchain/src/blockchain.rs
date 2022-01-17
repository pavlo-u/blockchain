

use rand::Rng;
use std::clone::Clone;
use std::collections::LinkedList; //blockchain
use std::collections::VecDeque; //queue
use std::fs;
use std::fs::File;
use std::io::prelude::Read;
use std::time::{Duration, Instant}; //time for create new block
use std::vec::Vec; //vector of Blockchains

pub mod block;
pub mod header;
pub mod timestamp;
pub mod transaction;
use crate::block::Block;
use crate::timestamp::current_timestamp;
use crate::transaction::Transaction;
use header::Header;
use borsh::{BorshSerialize, BorshDeserialize};

pub trait Backup {
    fn save(&self, path: String) -> std::io::Result<()>;
    fn load(path: String) -> Result<Blockchain, Box<dyn std::error::Error + 'static>>;
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Blockchain {
    pub blocks: LinkedList<Block>,
    pub transactions_queue: VecDeque<Transaction>,
}

impl Backup for Blockchain {
    fn save(&self, path: String) -> std::io::Result<()> {
        fs::write(path, self.try_to_vec().expect("Can`t serialize!"))
    }
    fn load(path: String) -> Result<Blockchain, Box<dyn std::error::Error + 'static>> {
        let mut file = File::open(path)?;
        let mut decoded = Vec::<u8>::new();
        file.read_to_end(&mut decoded).expect("Read_to_end is failed!");
        Ok(Blockchain::try_from_slice(&decoded).expect("try_to_slice is failed!"))
    }
}

impl Blockchain {
    //ctor
    pub fn new() -> Blockchain {
        //create timestamp
        //genesis block
        let transac_queue = VecDeque::new(); //create queue
        let mut blocks_list: LinkedList<Block> = LinkedList::new(); //create List
        let genesis = Block {
            head: Header {
                head_timestamp: current_timestamp(),
                nonce: rand::thread_rng().gen(), //random number
            },
            transaction: Transaction {
                from: "Genesis".to_string(),
                to: "".to_string(),
                amount: 0,
            },
            hash: current_timestamp(),
            previous_hash: "".to_string(),
        }; //create genesis block
        blocks_list.push_back(genesis);
        Blockchain {
            blocks: blocks_list,
            transactions_queue: transac_queue,
        }
    }
    pub fn new_transaction(&mut self, from: String, to: String, amount: u64) {
        self.transactions_queue
            .push_back(Transaction { from, to, amount });
    }
    pub fn mint(&mut self) {
        assert!(!self.transactions_queue.is_empty(), "Queue is empty!");//check queue
        
        let last_block = self.blocks.back().unwrap(); //last element of the list
        let mut mint_block = Block {
            //create block
            head: Header {
                head_timestamp: String::from(""),           //some timestamp
                nonce: rand::thread_rng().gen(), //some number
            },
            transaction: self.transactions_queue.pop_front().unwrap().clone(), //first transaction from the queue
            hash: current_timestamp(),                                         //some hash
            previous_hash: last_block.hash.to_string(), //hash of the last element
        };
        loop {
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
    pub fn fork_chain(&mut self, duration: u64) {
        let duration = Duration::new(duration, 0);
        let time_new_block = Duration::new(1, 0); //1 sec
                                                  //let time_three = Duration::new(3, 0);//3 sec
        let time_new_fork = Duration::new(5, 0); //5 sec
        let time_longest = Duration::new(37, 0); //37 sec
        let mut loop_duration; //time to break loop
        let mut block_duration; //time to create point
        let mut fork_duration; //time to create fork
        let mut longest_chain_duration: Duration; //time to choice longest chain
        let mut vec_chains: Vec<Blockchain> = Vec::new(); //vec of Blockchains
        let mut zero_blockchain: Blockchain = Blockchain {
            blocks: LinkedList::new(),
            transactions_queue: VecDeque::new(),
        };
        zero_blockchain
            .blocks
            .push_back(self.blocks.back().unwrap().clone());
        vec_chains.push(zero_blockchain);
        let mut rng = rand::thread_rng();
        let mut rand_num_of_bkch; //rand num in vec.len range
        let mut tmp_rand_num: usize; //create rand num
        let time_loop_stop = Instant::now();
        let mut time_block_create = Instant::now(); //time point to add block
        let mut time_block_fork = Instant::now(); //time point to add fork
        let mut time_block_chains = Instant::now(); //time point to select the longest chain
        loop {
            loop_duration = time_loop_stop.elapsed();
            if loop_duration >= duration {
                break;
            }
            block_duration = time_block_create.elapsed();
            if block_duration >= time_new_block {
                tmp_rand_num = rng.gen(); //create rand num
                if vec_chains.len() > 1 {
                    //add in rand Blockchain new block
                    rand_num_of_bkch = tmp_rand_num % vec_chains.len(); //get random number blockchain from vec range
                    vec_chains[rand_num_of_bkch].new_transaction(
                        "Test".to_string(),
                        "This".to_string(),
                        tmp_rand_num as u64,
                    ); //create new transaction in rand blockchain
                    vec_chains[rand_num_of_bkch].mint(); //create new block in rand blockchain
                } else {
                    self.new_transaction(
                        "Test".to_string(),
                        "This".to_string(),
                        tmp_rand_num as u64,
                    ); //create new transaction in main blockchain
                    self.mint(); //create new block in rand blockchain
                }
                time_block_create = Instant::now(); //reset time counter
            }
            fork_duration = time_block_fork.elapsed();
            if fork_duration >= time_new_fork {
                //add new fork
                let len_before_fork = vec_chains.len();
                tmp_rand_num = rng.gen(); //create rand num
                rand_num_of_bkch = tmp_rand_num % len_before_fork; //get random number blockchain from vec range
                vec_chains.push(vec_chains[rand_num_of_bkch].clone()); //create new (clone) blockchain and push
                vec_chains[len_before_fork].new_transaction(
                    "Fork".to_string(),
                    "Ff".to_string(),
                    tmp_rand_num as u64,
                ); //change current chain
                vec_chains[len_before_fork].mint(); //create new block
                time_block_fork = Instant::now(); //reset time counter
            }
            longest_chain_duration = time_block_chains.elapsed();
            if longest_chain_duration >= time_longest {
                let mut chain_lenght = 0;
                let mut i = 0;
                while vec_chains.len() > i {
                    //get longest
                    if vec_chains[i].blocks.len() > chain_lenght {
                        chain_lenght = vec_chains[i].blocks.len();
                    }
                    i += 1;
                }
                i = 0;
                while vec_chains.len() > i {
                    //remove the short
                    if chain_lenght > vec_chains[i].blocks.len() {
                        vec_chains.remove(i);
                    } else {
                        i += 1;
                    }
                }
                if vec_chains.len() == 1 {
                    //if only 1 chain is longest
                    vec_chains[0].blocks.pop_front();
                    self.blocks.append(&mut vec_chains[0].blocks); //add longest to the main chain
                    self.transactions_queue
                        .append(&mut vec_chains[0].transactions_queue);
                    vec_chains.clear();
                    zero_blockchain = Blockchain {
                        blocks: LinkedList::new(),
                        transactions_queue: VecDeque::new(),
                    };
                    zero_blockchain
                        .blocks
                        .push_back(self.blocks.back().unwrap().clone());
                    vec_chains.push(zero_blockchain);
                }
                time_block_chains = Instant::now(); //reset time counter
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
   
    #[test]
    fn new_blockchain_test() {
        let some_blockch: Blockchain = Blockchain::new();
        assert_eq!(some_blockch.blocks.back().expect("No blocks")
                .transaction.from.clone(), String::from("Genesis"));
        assert_eq!(some_blockch.blocks.len(), 1);
    }
    #[test]
    fn new_transaction_test() {
        let mut some_blockch: Blockchain = Blockchain::new();
        some_blockch.new_transaction("TestSender".to_string(), "Recipient".to_string(), 27);
        assert_eq!(some_blockch.transactions_queue.back().expect("No transactions")
                .from.clone(), String::from("TestSender"));
        assert_eq!(some_blockch.transactions_queue.len(), 1);
    }
    #[test]
    fn mint_test() {
        let mut some_blockch: Blockchain = Blockchain::new();
        some_blockch.transactions_queue.push_back( 
        Transaction {
            from: "Mint".to_string(),
            to: "Test".to_string(),
            amount: 0,
        });
        some_blockch.mint();
        assert_eq!(some_blockch.blocks.back().expect("No blocks")
                .transaction.to.clone(), String::from("Test"));
        assert_eq!(some_blockch.blocks.len(), 2);
    }
    #[test]
    fn fork_test() {
        let mut some_blockch: Blockchain = Blockchain::new();
        let blocks = some_blockch.blocks.len();
        let duration_sec: u64 = 100;
        some_blockch.fork_chain(duration_sec);
        assert_ne!(blocks, some_blockch.blocks.len());
    }
    #[test]
    fn save_test() {
        let path = String::from("Test");
        let mut some_blockch: Blockchain = Blockchain::new();
        some_blockch.blocks.push_back(Block {
            head: Header {
                head_timestamp: current_timestamp(),
                nonce: rand::thread_rng().gen(),
            },
            transaction: Transaction {
                from: "save".to_string(),
                to: "Test".to_string(),
                amount: 0,
            },
            hash: current_timestamp(),
            previous_hash: some_blockch.blocks.back().unwrap().hash.clone(),
        });
        some_blockch.save(path.clone()).expect("Can`t save file!");
        let serialize = some_blockch.try_to_vec().expect("Can`t serialize!");
        let some_data = fs::read(path).expect("Can`t open file!");
        assert_eq!(serialize, some_data);
    }
    #[test]
    fn load_test() {
        let path = String::from("Test");
        let mut some_blockch: Blockchain = Blockchain::new();
        some_blockch.blocks.push_back(Block {
            head: Header {
                head_timestamp: current_timestamp(),
                nonce: rand::thread_rng().gen(),
            },
            transaction: Transaction {
                from: "load".to_string(),
                to: "Test".to_string(),
                amount: 0,
            },
            hash: current_timestamp(),
            previous_hash: some_blockch.blocks.back().unwrap().hash.clone(),
        });
        fs::write(&path, some_blockch.try_to_vec()
                .expect("Can`t serialize!"))
                .expect("Can`t write!");
        let test_blockch = Blockchain::load(path).expect("Cant load file!");
        assert_eq!(some_blockch, test_blockch);
    }
}
