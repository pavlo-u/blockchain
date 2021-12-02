use sha2::{Sha256, Digest}; //hash
use std::collections::LinkedList; //blockchain
use std::collections::VecDeque; 
use std::time::{SystemTime, UNIX_EPOCH};//timestamp
#[derive(Debug, Clone, Copy)]
struct Transaction 
{
    from: String, 
    to: String, 
    amount: u64
}
#[derive(Debug, Clone, Copy)]
struct Block
{
    transaction: Transaction,
    hash: String
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

        let mut trnsctn = self.transaction.from + &self.transaction.to; //from + to
        trnsctn = trnsctn + &self.transaction.amount.to_string();//from+to + amount
        //trnsctn = trnsctn + &self.hash;//from+to+amount + previous hash
        let mut hasher = Sha256::new();
        hasher.update(trnsctn);
        format!("{:x}", hasher.finalize())
    }
}
#[derive(Debug)]
struct Blockchain
{
    blocks: LinkedList<Block>, 
    transactions_queue: VecDeque<Transaction> //очередь на обработку
}

impl Blockchain
{
    //ctor
    fn new() -> Blockchain {
        //create timestamp
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = since_the_epoch.as_millis().to_string();
        //genesis block
        let transac_queue = VecDeque::new(); //create queue
        let mut bloks_list: LinkedList<Block> = LinkedList::new();//create List
        let genesis = Block { 
            transaction: Transaction {
            from: "".to_string(), 
            to: "".to_string(), 
            amount: 0,
            }, 
            hash: timestamp 
        }; //create genesis block
        bloks_list.push_back(genesis);
        Blockchain { 
            blocks: bloks_list, 
            transactions_queue: transac_queue,
        }
    }
    /*
    fn new_transaction(from: String, to: String, amount: u64, trans_queue: &mut VecDeque<Transaction>){
        trans_queue.push_back(Transaction{
            from,
            to,
            amount
        }); 
    }
    fn new_block(&mut self, trans_queue: &mut VecDeque<Transaction>) {
        let last_elem = self.blocks.back().unwrap();//last element of the list
        let block = Block {
            transaction: self.transactions_queue.pop_front().unwrap(),
            hash: last_elem.hash.to_string()
        };
        self.blocks.push_back(block);
    }
    */
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
                hash: last_block.hash.to_string()//last element hash
            };
            block.hash += &block.hash_func(); //add the transaction hash
            self.blocks.push_back(block);
        }
    }
}

fn main()
{

   /* let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch.as_millis());
    let x = since_the_epoch.as_millis().to_string();
    println!("{}", x);*/
    let tr = Transaction {
        from: "Me".to_string(),
        to: "You".to_string(),
        amount: 11
    };
    let mut b = Block {
        transaction: tr,
        hash: "x".to_string()
    };
  //  println!("{:?}", &b);
    let tr = Transaction {
        from: "Me".to_string(),
        to: "You".to_string(),
        amount: 11
    };
  //  println!("\n{}\n", b.hash);
    b.hash = b.hash_func();
   // println!("\n{}\n", &b.hash);
   let mut blockch: Blockchain = Blockchain::new();
   blockch.new_transaction("Him".to_string(), "Her".to_string(), 11);
   blockch.new_block();
   println!("{:?}", blockch);
   
}