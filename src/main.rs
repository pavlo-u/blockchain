use sha2::{Sha256, Digest}; //hash
use std::collections::LinkedList; //blockchain
use std::collections::VecDeque; //queue
use std::time::{SystemTime, UNIX_EPOCH};//timestamp
/*////////////////////////////////
Blockchain emulation
*/////////////////////////////////
#[derive(Debug, Clone)]
struct Transaction {
    from: String, 
    to: String, 
    amount: u64
}
#[derive(Debug)]
struct Block {
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
        let timestamp = current_timestamp();
        //genesis block
        let transac_queue = VecDeque::new(); //create queue
        let mut bloks_list: LinkedList<Block> = LinkedList::new();//create List
        let genesis = Block { 
            transaction: Transaction {
            from: "Genesis".to_string(), 
            to: "".to_string(), 
            amount: 0,
            }, 
            hash: timestamp,
            previous_hash: "".to_string()
        }; //create genesis block
        bloks_list.push_back(genesis);
        Blockchain { 
            blocks: bloks_list, 
            transactions_queue: transac_queue,
        }
    }

    fn new_transaction(&mut self, from: String, to: String, amount: u64){
        self.transactions_queue.push_back(Transaction{
            from,
            to,
            amount
        }); 
    }

    fn new_block(&mut self) {
        if self.transactions_queue.is_empty() {
            println!("Transactions queue is empty!");
        } else {
            let last_block = self.blocks.back().unwrap();//last element of the list
            let first_tr = self.transactions_queue.pop_front().unwrap();//first element of the queue
            let mut block = Block { //create block
                transaction: first_tr,
                hash: last_block.hash.to_string(), //last element hash - any initialization
                previous_hash: last_block.hash.to_string()//last element hash
            };
          // let hash_tr = &block.hash_func();
          // block.hash += hash_tr; //add the transaction hash
             block.hash = block.hash_func(); //initialization hash
             self.blocks.push_back(block); //adding a new block to the list
        }
    }
}

fn main() {
    let mut blockch: Blockchain = Blockchain::new(); //create Blockchain
    println!("{:?}\n", blockch);
  
    blockch.new_transaction("H1".to_string(), "H1".to_string(), 33); 
    blockch.new_block();
    println!("{:?}\n", blockch);
    blockch.new_transaction("H2".to_string(), "H2".to_string(), 44); 
    blockch.new_block();
    println!("{:?}\n", blockch);
    

   
}

