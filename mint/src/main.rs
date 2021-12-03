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
        if self.transactions_queue.is_empty() {
            println!("Transactions queue is empty!");
            return;
        }
        else {
            let mut mint_block = Block { 
                head: Header {
                head_timestamp: current_timestamp(),
                nonce: rand::thread_rng().gen_bigint(1000) //random number
                },
                transaction: Transaction {
                from: "".to_string(), 
                to: "".to_string(), 
                amount: 0,
                }, 
                hash: current_timestamp(),
                previous_hash: "".to_string()
            };            
            loop {
                mint_block.head.nonce = rand::thread_rng().gen_bigint(1000);
                let tmp_hash = mint_block.hash_func();
                let v: Vec<&str> = tmp_hash.matches("1").collect();
                println!("{}\n\n", tmp_hash);
                if v.len() >= 6 {
                    mint_block.hash = tmp_hash;
                    break;
                }
            }
        }              
    }

}

fn main() {
    let mut blockch: Blockchain = Blockchain::new(); //create Blockchain
    println!("{:?}\n\n", blockch);

    blockch.new_transaction("H1".to_string(), "H1".to_string(), 33);
    blockch.mint();
    println!("{:?}\n\n", blockch);

   
}


