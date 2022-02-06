#[path = "../blockchain/blockchain.rs"]
pub mod blockchain;
#[path = "../blockchain/server.rs"]
pub mod server;

use actix_web::error::PayloadError;
use blockchain::{block::transaction::Transaction, block::Block, Blockchain};
use server::Node;

use actix_web::{
    client::Client, error, get, web, web::Bytes, App, Error, HttpResponse, HttpServer,
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
            .service(first_page)
            .service(node_status)
            .service(get_blockchain)
            .service(genesis_generate)
            .service(web::resource("v1/private/blocks/new").route(web::post().to(add_block)))
            .service(new_node)
            .service(node_config)
            .service(start_working)
            .service(nodes_watch)
            //.service(consensus)
    })
    .bind(addr)
    .expect("Check bind path!")
    .run()
    .await?;

    Ok(())
}
async fn genesis_check(some_config: &HashSet<String>, len_blnch: usize) {
    if len_blnch == 0 {
        for node in some_config {
            match Client::new()
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
    } else {
        println!("Genesis exist!");
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
    let some_rand_num: usize = rand_num % 5 + 1; //mint time
    let t_block_mint = Duration::new(some_rand_num as u64, 0);
    println!("{:?}", t_block_mint);
    let len_blnch = some_blockch
        .lock()
        .expect("Can`t check genesis")
        .blocks
        .len();
    let some_config = some_node
        .lock()
        .expect("Can`t send mint block!")
        .config_list
        .clone();

    genesis_check(&some_config, len_blnch).await;

    loop {
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
        //no forks
        if number_of_forks == 0 {
            some_blockch.lock().expect("Can`t mint!").mint(mempool_part);
            let block = some_blockch
                .lock()
                .expect("Can`t get mint block!")
                .blocks
                .back()
                .expect("Can`t get mint block! Block does not exist!")
                .clone();
            send_block(&some_config, block).await;
        } else {
            //with forks
            let rand_fork: usize = rng.gen();
            let rand_fork: usize = rand_fork % number_of_forks;
            //get random fork (blockchain)
            let mut rand_fork: Blockchain = some_forks
                .lock()
                .expect("Can`t mint inside the fork!")
                .remove(rand_fork);
            //mint new block
            rand_fork.mint(mempool_part);
            //get mint block
            let block = rand_fork
                .blocks
                .back()
                .expect("Can`t get mint block in the fork! Block does not exist!")
                .clone();
            //put random fork back
            some_forks
                .lock()
                .expect("Can`t put the fork back!")
                .push(rand_fork);
            //send mint block to other nodes
            send_block(&some_config, block).await;
        }
    }

    //Ok(HttpResponse::Ok().json(format!("My watch has ended!",)))
}

async fn send_block(some_config: &HashSet<String>, block: Block) {
    for node in some_config {
        let url = String::from("http://") + node + "/v1/private/blocks/new";
        match Client::new().post(url).send_json(&block.clone()).await {
            Ok(_) => println!("Sending was successful! {:?}", std::thread::current()),
            Err(e) => println!("Sending wasn`t successful! Error:\n{:?}", e),
        };
    }
}

#[get("/v1/start")]
async fn start_working(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    let mut config: HashSet<String> = some_node.lock().expect("Can`t start!").config_list.clone();
    let self_addr = some_node.lock().expect("Can`t start!").addr.clone();
    let self_full_addr = self_addr + &some_node.lock().expect("Can`t start!").port;
    config.insert(self_full_addr);
    for node in config {
        let url = String::from("http://") + &node + "/v1/";
        let client = Client::new();
        match client.get(url).send().await {
            Ok(_) => println!("{} starts to work!", &node),
            Err(e) => println!("Start error!\n{} will not work! Error:\n{:?}", &node, e),
        };
    }
    //Client::new().get(String::from("http://") + &some_node.lock().expect("Can`t start!").addr.clone() + &some_node.lock().expect("Can`t start!").port +  "/v1/").send().await;
    Ok(HttpResponse::Ok().json(format!("Yes, my lord!")))
}
async fn add_block(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
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
    //no forks
    let number_of_forks = some_forks.lock().expect("Can`t create fork!").len();
    if number_of_forks == 0 {
        let last_block = some_blockch
            .lock()
            .expect("Can`t add block!")
            .blocks
            .pop_back();
        match last_block {
            None => {
                if some_block.previous_hash == String::from("Genesis has no previous hash") {
                    println!("Adding a genesis block!");
                    some_blockch
                        .lock()
                        .expect("Can`t add block!")
                        .blocks
                        .push_back(some_block);
                    return Ok(
                        HttpResponse::Ok().json(format!("Adding a genesis block successfully!"))
                    );
                } else {
                    return Err(error::ErrorBadRequest("Create a genesis block first!"));
                }
            }
            Some(block) => {
                if block.hash == some_block.previous_hash {
                    //put back
                    some_blockch
                        .lock()
                        .expect("Can`t put back block!")
                        .blocks
                        .push_back(block);
                    //just add a block
                    some_blockch
                        .lock()
                        .expect("Can`t add block!")
                        .blocks
                        .push_back(some_block);
                    return Ok(HttpResponse::Ok().json(format!("Adding a block successfully!")));
                } else if block.previous_hash == some_block.previous_hash {
                    //create 2 forks
                    println!("\nCreate fork!\n");
                    let mut fork1: Blockchain = Blockchain {
                        blocks: LinkedList::new(),
                    };
                    fork1.blocks.push_back(some_block);
                    some_forks
                        .lock()
                        .expect("Can`t push block in fork!")
                        .push(fork1);
                    let mut fork2: Blockchain = Blockchain {
                        blocks: LinkedList::new(),
                    };
                    fork2.blocks.push_back(block);
                    some_forks
                        .lock()
                        .expect("Can`t push last block in fork!")
                        .push(fork2);
                    // println!(
                    //     "\nBlnch\n{:#?}\n\nForks\n{:#?}\n\n",
                    //     &some_blockch, &some_forks
                    // );
                    return Ok(HttpResponse::Ok().json(format!("There has been a network fork!")));
                } else {
                    //put back
                    some_blockch
                        .lock()
                        .expect("Can`t put back block!")
                        .blocks
                        .push_back(block);
                    println!(
                        "\nBlnch\n{:?}\n\nForks\n{:?}\n\nBlock\n{:?}\n",
                        &some_blockch, &some_forks, &some_block
                    );
                    println!("Wrong previous hash!");
                    //invalid block
                    return Err(error::ErrorBadRequest("The hashes don't match!"));
                }
            }
        }
    } else {
        //with forks
        for chain in 0..number_of_forks {
            let current_fork_last_block = some_forks
                .lock()
                .expect("Can't match a block")
                .get(chain)
                .expect("Can`t some get fork!")
                .blocks
                .back()
                .expect("Can`t get last block from fork!")
                .clone();
            if current_fork_last_block.hash == some_block.previous_hash {
                some_forks
                    .lock()
                    .expect("Can't add a block to the fork")
                    .get_mut(chain)
                    .expect("Can`t get fork!")
                    .blocks
                    .push_back(some_block);
                consensus(&some_forks, &some_blockch).await;
                return Ok(HttpResponse::Ok().json(format!("Adding a block is successful!")));
            } else if current_fork_last_block.previous_hash == some_block.previous_hash {
                 //create fork in forks
                 println!("\nCreate forks  fork!\n");
                 let mut fork: Blockchain = Blockchain {
                     blocks: some_forks
                     .lock()
                     .expect("Can`t push block in forks fork!")
                     .get(chain)
                     .expect("Can`t get fork from vec!")
                     .blocks
                     .clone(),
                 };
                 fork.blocks.pop_back();
                 fork.blocks.push_back(some_block);
                 some_forks
                     .lock()
                     .expect("Can`t push fors in vec forks!")
                     .push(fork);
                
                //  println!(
                //      "\nBlnch\n{:#?}\n\nForks\n{:#?}\n\n",
                //      &some_blockch, &some_forks
                //  );
                 return Ok(HttpResponse::Ok().json(format!("There has been a network fork!")));
            }
        }
        println!(
            "\nBlnch\n{:#?}\n\nForks\n{:#?}\n\nBlock with forks\n{:#?}\n\n",
            &some_blockch, &some_forks, &some_block
        );
        println!("Wrong previous hash!!!!!!!!");
        
        return Err(error::ErrorBadRequest("The  hashes don't match!!!!!!!!"));
    }
    // Ok(HttpResponse::Ok().json(format!("Adding a block successfully!")))
}

//#[get("/v1/private/consensus/")]
async fn consensus(
    some_forks: &web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    some_blockch: &web::Data<Arc<Mutex<Blockchain>>>,
) /*-> Result<HttpResponse, Error>*/ {
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
                .get(fork)
                .expect("Can't get a block to append!")
                .blocks
                .clone();
            some_blockch
                .lock()
                .expect("Can`t append block!")
                .blocks
                .append(&mut valid_fork);
            some_forks.lock().expect("Can't clear forks!").clear();
            println!("A consensus has been found!");
            //println!("Blnch\n{:#?}\nForks\n{:#?}\n", &some_blockch, &some_forks);
            return;
         //   return Ok(HttpResponse::Ok().json(format!("A consensus has been found!")));
        }
    }
  //  Ok(HttpResponse::Ok().json(format!("Not enough blocks in forks!")))
}

#[get("/v1/private/")]
async fn get_blockchain(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
) -> Result<HttpResponse, Error> {
    let some_blockch = some_blockch.lock().expect("Can`t get blockchain!");
    let some_blockchain = Blockchain {
        blocks: some_blockch.blocks.clone(),
    };
    println!("Some_forks!\n\n{:#?}\n", some_forks);
    Ok(HttpResponse::Ok().json(some_blockchain))
}

#[get("/v1/private/genesis/")]
async fn genesis_generate(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_node: web::Data<Arc<Mutex<Node>>>,
) -> Result<HttpResponse, Error> {
    //if genesis already exists
    let chain_len = some_blockch
        .lock()
        .expect("Can`t create genesis!")
        .blocks
        .len();
    if chain_len != 0 {
        Err(error::ErrorBadRequest("Genesis block already exists!"))
    } else {
        let genesis_block = Blockchain::new()
            .blocks
            .pop_back()
            .expect("Can`t generate genesis!");
        some_blockch
            .lock()
            .expect("Can`t add genesis block!")
            .blocks
            .push_back(genesis_block.clone());
        let uri_list = some_node
            .lock()
            .expect("Can`t send genesis!")
            .config_list
            .clone();
        for node in uri_list {
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
                //launching a node 
               /* * let self_addr =
                    url.clone() + &some_node.lock().expect("Can`t get current config!").addr;
                let self_addr =
                    self_addr + &some_node.lock().expect("Can`t get current config!").port;
                match client.get(self_addr.clone() + "/v1/").send().await {
                    Ok(_) => println!("The node {} was launched!", &self_addr),
                    Err(_) => return Err(error::ErrorBadGateway("The node was not running!")),
                };*/
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

#[get("/")]
async fn first_page() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(format!(
        "Hey! Go try:
    Get
    /v1/public/transactions/:  and some index/n
    /v1/public/blocks/:  and some index
    /v1/public/blocks/head
    POST
    v1/public/transactions/new"
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
