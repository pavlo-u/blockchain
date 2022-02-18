#[path = "../blockchain/blockchain.rs"]
pub mod blockchain;
#[path = "../blockchain/server.rs"]
pub mod server;

use blockchain::{block::transaction::Transaction, block::Block, Blockchain};
use server::Node;

use actix_web::{
    client::Client, error, error::PayloadError, get, web, web::Bytes, App, Error, HttpResponse,
    HttpServer,
};
use futures::StreamExt;
use rand::Rng;
use std::collections::{HashSet, LinkedList};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[actix_web::main]
pub async fn blockchain_network(some_path: String) -> Result<(), std::io::Error> {
    let addr = String::from("127.0.0.1:80");
    let some_node = Node::new(addr.clone(), some_path.clone());

    println!("{:?}", &some_node);
    // println!("{:?}", std::str::from_utf8(&some_node.id.as_slice()).unwrap());
    let addr = addr + &some_path;
    let mempool = web::Data::new(Arc::new(Mutex::new(Vec::<Transaction>::new())));
    let forks_state = web::Data::new(Arc::new(Mutex::new(Vec::<Blockchain>::new())));
    let state_server = web::Data::new(Arc::new(Mutex::new(some_node)));
    let state_blnch = web::Data::new(Arc::new(Mutex::new(Blockchain {
        blocks: LinkedList::new(),
    })));

    println!("Starting http server: http://{}/\n", &addr);
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::clone(&state_blnch))
            .app_data(web::Data::clone(&state_server))
            .app_data(web::Data::clone(&forks_state))
            .app_data(web::Data::clone(&mempool))
            .service(
                web::resource("v1/public/transactions/new").route(web::post().to(add_transactions)),
            )
            .service(view_trans)
            .service(view_block_by_index)
            .service(view_top)
            .service(web::resource("v1/private/node/change").route(web::post().to(change_config)))
            .service(node_status)
            .service(get_blockchain)
            .service(genesis_generate)
            .service(web::resource("v1/private/blocks/new").route(web::post().to(add_block)))
            .service(new_node)
            .service(node_config)
            .service(start_working)
            .service(nodes_watch)
    })
    .workers(2)
    .bind(addr)
    .expect("Check bind path!")
    .run()
    .await
}
async fn send_transaction(some_config: &HashSet<String>, some_transaction: Transaction) {
    for node in some_config {
        let url: String = String::from("http://") + node + "/v1/public/transactions/new";
        let transaction: Transaction = some_transaction.clone();
        let client = Client::new();
        actix_web::rt::spawn(async move {
            match client.post(url).send_json(&transaction.clone()).await {
                Ok(_) => ()/*println!(
                    "Sending transaction was successful {:?}!",
                    std::thread::current()
                )*/,
                Err(e) => println!(
                    "Alert! Sending transaction wasn`t successful! Error:\n{:?}",
                    e
                ),
            };
        });
    }
}
async fn gen_transactions_loop(some_config: HashSet<String>) {
    let mut rng = rand::thread_rng();
    loop {
        async_std::task::sleep(Duration::from_secs(1)).await;
        let transaction: Transaction = Transaction {
            from: rng.gen::<u64>().to_string(),
            to: rng.gen::<u64>().to_string(),
            amount: rng.gen::<u64>(),
        };
        send_transaction(&some_config, transaction).await;
    }
}

async fn genesis_create(some_config: &HashSet<String>) {
    for node in some_config {
        let client = Client::new();
        match client
            .get(String::from("http://") + node + "/v1/private/genesis/")
            .send()
            .await
        {
            Ok(_) => {
                println!("Genesis block established successfully!");
                break;
            }
            Err(e) => println!("Something wrong! Error {:?}", e),
        }
    }
}
#[get("/v1/")]
async fn nodes_watch(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_node: web::Data<Arc<Mutex<Node>>>,
    some_mempool: web::Data<Arc<Mutex<Vec<Transaction>>>>,
    some_forks: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
) -> Result<HttpResponse, Error> {
    println!("Start work! ");

    let mut rng = rand::thread_rng();
    let rand_num: usize = rng.gen();
    let some_rand_num: usize = rand_num % 9 + 6; //mint time
    let t_block_mint = Duration::from_secs(some_rand_num as u64);
    println!("{:?}", t_block_mint);
    let mut some_config = some_node
        .lock()
        .expect("Can`t send mint block!")
        .config_list
        .clone();
    let self_addr = some_node.lock().expect("Can`t start!").addr.clone();
    let self_full_addr = self_addr + &some_node.lock().expect("Can`t start!").port;
    some_config.insert(self_full_addr);
    actix_web::rt::spawn(gen_transactions_loop(some_config.clone()));
    actix_web::rt::spawn(async move {
        loop {
            println!("Sleep {:?}!", t_block_mint);
            async_std::task::sleep(t_block_mint).await;
            let mut tr_count = some_mempool
                .lock()
                .expect("Can`t get transactions count")
                .len();
            if tr_count > some_rand_num {
                tr_count = some_rand_num; //transaction count = mint time
            }
            let mempool_part: Vec<Transaction> = some_mempool
                .lock()
                .expect("Can`t get mempool!")
                .drain(..tr_count)
                .collect();
            let number_of_forks = some_forks.lock().expect("Can`t get number of forks!").len();

            if number_of_forks == 0 {
                //no forks
                let mint_block: Block =
                    some_blockch.lock().expect("Can`t mint!").mint(mempool_part);
                send_block(&some_config, mint_block).await;
            } else {
                //with forks
                let rand_fork: usize = rng.gen();
                let rand_fork: usize = rand_fork % number_of_forks;
                let mint_block: Block = some_forks
                    .lock()
                    .expect("Can`t mint inside the fork!")
                    .get_mut(rand_fork)
                    .expect("Can`t get fork for mint!")
                    .mint(mempool_part);
                send_block(&some_config, mint_block).await;
                consensus(&some_forks, &some_blockch).await;
            }
        }
    });
    Ok(HttpResponse::Ok().json(format!("I`m working!")))
}
async fn send_block(some_config: &HashSet<String>, block: Block) {
    for node in some_config {
        let some_block = block.clone();
        let client = Client::new();
        let url = String::from("http://") + node + "/v1/private/blocks/new";
        actix_web::rt::spawn(async move {
            match client.post(url).send_json(&some_block.clone()).await {
                Ok(_) => (), //println!("Sending block was successful! {:?}", std::thread::current()),
                Err(e) => println!(
                    "Sending block wasn`t successful!\nBlock {:?}\nError:\n{:?}",
                    some_block, e
                ),
            };
        });
    }
}

#[get("/v1/start")]
async fn start_working(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    let mut some_config: HashSet<String> =
        some_node.lock().expect("Can`t start!").config_list.clone();
    genesis_create(&some_config).await;
    let self_addr = some_node.lock().expect("Can`t start!").addr.clone();
    let self_full_addr = self_addr + &some_node.lock().expect("Can`t start!").port;

    some_config.insert(self_full_addr);
    let client = Client::new();
    for node in some_config {
        let url = String::from("http://") + &node + "/v1/";
        match client.get(url).send().await {
            Ok(_) => println!("{} starts to work!", node),
            Err(some_error) => println!("Start error!\n{} send error:\n{:?}", node, some_error),
        };
    }
    Ok(HttpResponse::Ok().json(format!("Yes, my lord!")))
}
async fn delete_mempool_trans(
    some_mempool: &web::Data<Arc<Mutex<Vec<Transaction>>>>,
    some_block: &Block,
) {
    for i in 0..some_block.transaction.len() {
        some_mempool
            .lock()
            .unwrap()
            .retain(|mempool_tr| mempool_tr != &some_block.transaction[i]);
    }
}
async fn add_block(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_mempool: web::Data<Arc<Mutex<Vec<Transaction>>>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > 262_144 {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let some_block = serde_json::from_slice::<Block>(&body)?;
    delete_mempool_trans(&some_mempool, &some_block).await;
    let no_forks = some_forks.lock().expect("Can`t create fork!").is_empty();
    if no_forks {
        add_to_blockchain(&some_blockch, &some_forks, some_block).await
    } else {
        add_to_fork(&some_blockch, &some_forks, some_block).await
    }
}

async fn add_to_blockchain(
    some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_block: Block,
) -> Result<HttpResponse, Error> {
    let top_block = some_blockch
        .lock()
        .expect("Can`t check genesis!")
        .blocks
        .back()
        .cloned();
    match top_block {
        None => {
            if some_block.previous_hash == String::from("Genesis has no previous hash") {
                println!("Adding a genesis block!");
                some_blockch
                    .lock()
                    .expect("Can`t add block!")
                    .blocks
                    .push_back(some_block);
                return Ok(HttpResponse::Ok().json(format!("Adding a genesis block successfully!")));
            }
            println!("gen err");
            return Err(error::ErrorBadRequest("Create a genesis block first!"));
        }
        Some(current_last_block) => {
            if current_last_block.hash == some_block.previous_hash {
                //println!("Normal Current block {:?}", some_block);
                some_blockch
                    .lock()
                    .expect("Can`t add block!")
                    .blocks
                    .push_back(some_block);
                return Ok(HttpResponse::Ok().json(format!("Adding a block successfully!")));
            } else {
                create_fork(&some_blockch, &some_forks, some_block).await
            }
        }
    }
}
async fn create_fork(
    some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_block: Block,
) -> Result<HttpResponse, Error> {
    let len_blch: usize = some_blockch.lock().expect("Can`t lock!").blocks.len();
    let mut last_blocks: usize = 5; //desired number of blocks
    if last_blocks > len_blch {
        last_blocks = len_blch;
    }
    let len_blch: usize = len_blch - last_blocks;
    let mut some_last_blocks: LinkedList<Block> = some_blockch
        .lock()
        .expect("Can`t lock!")
        .blocks
        .split_off(len_blch);
    let mut iter = some_last_blocks.iter().rev();
    for counter in (0..last_blocks).rev() {
        let block: &Block = match iter.next() {
            Some(block) => block,
            None => {
                some_blockch
                    .lock()
                    .expect("Can`t lock!")
                    .blocks
                    .append(&mut some_last_blocks);
                //println!("Fork was not created!");
                break;
            }
        };
        if block.hash == some_block.previous_hash {
            let old_fork = some_last_blocks.split_off(counter);
            some_blockch
                .lock()
                .expect("Can`t lock for create fork!")
                .blocks
                .append(&mut some_last_blocks);
            //println!("Current fork after split {:?}", old_fork);
            let old_fork_chain = Blockchain { blocks: old_fork };
            let mut new_fork_chain = Blockchain {
                blocks: LinkedList::new(),
            };
            //println!("Fork Current block {:?}", some_block);
            new_fork_chain.blocks.push_back(some_block);
            some_forks
                .lock()
                .expect("Can`t create fork!")
                .push(old_fork_chain);
            some_forks
                .lock()
                .expect("Can`t create fork!")
                .push(new_fork_chain);
            return Ok(HttpResponse::Ok().json(format!("There has been a network fork!")));
        }
    }
    some_blockch
        .lock()
        .expect("Can`t lock!")
        .blocks
        .append(&mut some_last_blocks);
    //println!("ERROR! {:?}", some_block);
    return Err(error::ErrorBadRequest("Error! Fork was not created!"));
}
async fn add_to_fork(
    some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_block: Block,
) -> Result<HttpResponse, Error> {
    let number_of_forks: usize = some_forks.lock().expect("Can`t create fork!").len();
    //if match - add to fork
    for chain in 0..number_of_forks {
        let forks_top_block = some_forks
            .lock()
            .expect("Can't match a block")
            .get(chain)
            .expect("Can`t some get fork!")
            .blocks
            .back()
            .cloned();
        match forks_top_block {
            None => return Err(error::ErrorBadRequest("No blocks in fork!")),
            Some(last_block) => {
                if last_block.hash == some_block.previous_hash {
                    //println!("Add to fork {:?}", some_block);
                    some_forks
                        .lock()
                        .expect("Can't add a block to the fork")
                        .get_mut(chain)
                        .expect("No any fork here!")
                        .blocks
                        .push_back(some_block);
                    consensus(&some_forks, &some_blockch).await;
                    return Ok(HttpResponse::Ok().json(format!("Adding a block is successful!")));
                } else if last_block.previous_hash == some_block.previous_hash {
                    //create fork in forks
                    //println!("Create forked fork!{:?}", some_block);
                    let mut fork: Blockchain = some_forks
                        .lock()
                        .expect("Can`t push block in forked fork!")
                        .get(chain)
                        .expect("Can`t get fork from vec!")
                        .clone();
                    fork.blocks.pop_back();
                    fork.blocks.push_back(some_block);
                    some_forks
                        .lock()
                        .expect("Can`t push forks in vec forks!")
                        .push(fork);

                    return Ok(HttpResponse::Ok().json(format!("There has been a network fork!")));
                }
            }
        };
    }
    //else create fork from fork
    create_forked_fork(&some_forks, /*&some_blockch,*/ some_block).await
}

async fn create_forked_fork(
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    //some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
    some_block: Block,
) -> Result<HttpResponse, Error> {
    let number_of_forks: usize = some_forks.lock().expect("Can`t create fork!").len();
    for num_chain in 0..number_of_forks {
        let mut current_fork: LinkedList<Block> = some_forks
            .lock()
            .expect("Can't match a block")
            .get(num_chain)
            .expect("Can`t get some fork!!")
            .blocks
            .clone();
        let mut iter = current_fork.iter().rev();
        for iter_block in (0..current_fork.len()).rev() {
            match iter.next() {
                None => break,
                Some(current_block) => {
                    if current_block.hash == some_block.previous_hash {
                        //create fork in forks
                        //println!("Create forks fork!");
                        current_fork.split_off(iter_block);
                        current_fork.push_back(some_block);
                        let new_fork = Blockchain {
                            blocks: current_fork,
                        };
                        some_forks
                            .lock()
                            .expect("Can`t add fork forks to vec!")
                            .push(new_fork);
                        return Ok(HttpResponse::Ok()
                            .json(format!("There has been a network forks fork!")));
                    }
                }
            };
        }
    }
    /*println!(
        "Blch\n{:?}\nBlock\n{:?}",
        &some_blockch.lock().unwrap().blocks.back(),
        some_block
    );
    println!("No valid previous hash");*/
    Err(error::ErrorBadRequest("The  hashes don't match!!!!!!!!"))
}

async fn consensus(
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
) {
    let chain: usize = 5;
    let number_of_forks: usize = some_forks.lock().expect("Can't come to consensus!").len();
    for fork in 0..number_of_forks {
        if some_forks
            .lock()
            .expect("Can't add a block to the fork")
            .get(fork)
            .expect("Can`t get some fork!")
            .blocks
            .len()
            > chain
        {
            let mut valid_fork: LinkedList<Block> = some_forks
                .lock()
                .expect("Can't get a fork to append!")
                .get_mut(fork)
                .expect("Can't get a block to append!")
                .blocks
                .clone();
            some_blockch
                .lock()
                .expect("Can`t append block!")
                .blocks
                .append(&mut valid_fork);
            some_forks.lock().expect("Can't clear forks!").clear();
            /*println!(
                "\nA consensus has been found! Now last block {:?}\n",
                some_blockch.lock().unwrap().blocks.back()
            );*/
            return;
        }
    }
}

#[get("/v1/private/")]
async fn get_blockchain(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
) -> Result<HttpResponse, Error> {
    let some_blockch = some_blockch.lock().expect("Can`t get blockchain!");
    let some_blockchain = Blockchain {
        blocks: some_blockch.blocks.clone(),
    };
    Ok(HttpResponse::Ok().json(some_blockchain))
}

#[get("/v1/private/genesis/")]
async fn genesis_generate(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_node: web::Data<Arc<Mutex<Node>>>,
) -> Result<HttpResponse, Error> {
    if !some_blockch
        .lock()
        .expect("Can`t create genesis!")
        .blocks
        .is_empty()
    {
        return Err(error::ErrorBadRequest("Genesis block already exists!"));
    }
    let genesis_block = Blockchain::new()
        .blocks
        .pop_back()
        .expect("Can`t generate genesis!");
    some_blockch
        .lock()
        .expect("Can`t add genesis block!")
        .blocks
        .push_back(genesis_block.clone());
    let some_config = some_node
        .lock()
        .expect("Can`t send genesis!")
        .config_list
        .clone();
    println!("Config {:?}", some_config);
    for node in some_config {
        let path: String = String::from("http://") + &node + "/v1/private/blocks/new";
        let client = Client::new();
        match client.post(path).send_json(&genesis_block.clone()).await {
            Ok(_) => println!("Send genesis to {} successfully", &node),
            Err(e) => println!("Node {} \nError\n{} ", &node, e),
        };
    }
    Ok(HttpResponse::Ok().json(format!(
        "Genesis block created successfully! {:?}",
        genesis_block
    )))
}
#[get("/v1/private/node/new")]
async fn new_node(
    some_node: web::Data<Arc<Mutex<Node>>>,
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
) -> Result<HttpResponse, Error> {
    let list_nodes = some_node
        .lock()
        .expect("Can`t add new node!")
        .config_list
        .clone();
    let url: String = String::from("http://");
    let client = Client::new();
    //get current state from another node
    for node in list_nodes {
        let current_blockchain_byte = client
            .get(url.clone() + &node + "/v1/private/")
            .send()
            .await;
        let some_bytes: Result<Bytes, PayloadError>;
        //another node may not be deployed
        if let Ok(mut correct_path) = current_blockchain_byte {
            some_bytes = correct_path.body().await;
            //we may not get the blockchain state
            if let Ok(blockch_byte) = some_bytes {
                //serialize blockchain state
                let blockchain = serde_json::from_slice::<Blockchain>(&blockch_byte)
                    .expect("Can`t serialize blockchain!");
                //replace current blockchain state with new one
                let mut some_blockch = some_blockch.lock().expect("Can`t get blockchain!");
                some_blockch.blocks = blockchain.blocks.clone();
                //get the configuration of the other node
                let some_config_list = client
                    .get(url.clone() + &node + "/v1/public/node/status/config")
                    .send()
                    .await
                    .expect("Can`t get config!")
                    .body()
                    .await
                    .expect("Can`t get config list!");
                //serialize node state
                let mut some_config = serde_json::from_slice::<HashSet<String>>(&some_config_list)
                    .expect("Can`t serialize config!");
                //add to config deployed node
                some_config.insert(node.clone());
                //add addr the current node to the config for the rest of the nodes
                for var_node in &some_config {
                    let var_url = url.clone() + &var_node;
                    let another_node = some_node
                        .lock()
                        .expect("Can`t get current port!")
                        .port
                        .clone();
                    match client
                        .post(var_url.clone() + "/v1/private/node/change")
                        .send_json(&another_node)
                        .await
                    {
                        Ok(_) => println!("The node {} config list has been changed!", &var_url),
                        Err(e) => println!(
                            "The node {} config list has not been changed! {}",
                            &var_url, e
                        ),
                    };
                }
                some_node
                    .lock()
                    .expect("Can`t get current config!")
                    .config_list = some_config;

                return Ok(HttpResponse::Ok().json(format!("Node add to network successfully!")));
            }
            println!("Can`t get state from {}!", &node);
        } else {
            println!("Node {} is not deployed!", &node);
        }
    }
    Err(error::ErrorBadGateway(
        "The node has not been added to the network!",
    ))
}

async fn change_config(
    some_node: web::Data<Arc<Mutex<Node>>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > 262_144 {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let mut req_body = serde_json::from_slice::<String>(&body)?;
    req_body.insert_str(0, "127.0.0.1:80");
    let mut some_node = some_node.lock().expect("Can`t change config!");
    some_node.config_list.insert(req_body.clone());
    Ok(HttpResponse::Ok().json(format!(
        "A new address has been added for {} ",
        (some_node.addr.clone() + &some_node.port)
    )))
}
#[get("/v1/public/node/status/config")]
async fn node_config(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(
        some_node
            .lock()
            .expect("Can`t show status!")
            .config_list
            .clone(),
    ))
}
#[get("/v1/public/node/status")]
async fn node_status(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(format!(
        "{:?}",
        some_node.lock().expect("Can`t show status!")
    )))
}

#[get("/v1/public/blocks/head")]
async fn view_top(some_blockch: web::Data<Arc<Mutex<Blockchain>>>) -> Result<HttpResponse, Error> {
    let some_blockchain = some_blockch.lock().expect("Can`t get head!");
    Ok(HttpResponse::Ok().json(format!(
        "Top block is: {:#?} len {}",
        some_blockchain.blocks.back(),
        some_blockchain.blocks.len(),
    )))
}

async fn add_transactions(
    some_mempool: web::Data<Arc<Mutex<Vec<Transaction>>>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > 262_144 {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let some_tr: Transaction = serde_json::from_slice::<Transaction>(&body)?;
    some_mempool
        .lock()
        .expect("Can`t add transaction!")
        .push(some_tr);
    Ok(HttpResponse::Ok().json(format!("The transaction was added successfully!")))
}

#[get("/v1/public/transactions/:{index}")]
async fn view_trans(
    some_mempool: web::Data<Arc<Mutex<Vec<Transaction>>>>,
    web::Path(index): web::Path<usize>,
) -> Result<HttpResponse, Error> {
    let some_blockchain = some_mempool.lock().expect("Can`t get transaction!");
    Ok(HttpResponse::Ok().json(format!(
        "Transaction number {} is: {:#?} len {}",
        index,
        some_blockchain.get(index),
        some_blockchain.len()
    )))
}

#[get("/v1/public/blocks/:{index}")]
async fn view_block_by_index(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    web::Path(index): web::Path<usize>,
) -> Result<HttpResponse, Error> {
    let some_blockchain = some_blockch.lock().expect("Can`t get block!");
    Ok(HttpResponse::Ok().json(format!(
        "Block by number {} is: {:#?} len {}",
        index,
        some_blockchain.blocks.iter().nth(index),
        some_blockchain.blocks.len()
    )))
}
