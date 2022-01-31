pub mod blockchain;
pub mod server;
pub mod transaction;
use actix_web::error::PayloadError;
use blockchain::{block::Block, Blockchain};
use server::Node;
use transaction::Transaction;

use actix_web::{
    client::Client, error, get, web, web::Bytes, App, Error, HttpResponse, HttpServer,
};
use futures::StreamExt;
use rand::Rng;
use std::collections::{HashSet, LinkedList, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[actix_web::main]
pub async fn blockchain_network(some_path: String) -> Result<(), std::io::Error> {
    let addr = String::from("127.0.0.1:80");
    let some_node = Node::new(addr.clone(), some_path.clone());

    println!("{:?}", &some_node);
    // println!("{:?}", std::str::from_utf8(&some_node.id.as_slice()).unwrap());
    let addr = addr + &some_path;
    let forks_state = web::Data::new(Arc::new(Mutex::new(Vec::<LinkedList<Block>>::new())));
    let state_server = web::Data::new(Arc::new(Mutex::new(some_node)));
    let state_blnch = web::Data::new(Arc::new(Mutex::new(Blockchain {
        blocks: LinkedList::new(),
        transactions_queue: VecDeque::new(),
    })));

    println!("Starting http server: http://{}/\n", &addr);
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::clone(&state_blnch))
            .app_data(web::Data::clone(&state_server))
            .app_data(web::Data::clone(&forks_state))
            .service(
                web::resource("v1/public/transactions/new").route(web::post().to(add_transactions)),
            )
            .service(view_trans)
            .service(view_by_index)
            .service(view_top)
            .service(web::resource("v1/private/node/change").route(web::post().to(change_config)))
            .service(first_page)
            .service(node_status)
            .service(get_blockchain)
            .service(genesis_generate)
            .service(web::resource("v1/private/blocks/new").route(web::post().to(add_block)))
            .service(web::resource("v1/private/node/new").route(web::post().to(new_node)))
            .service(node_config)
            .service(start_working)
            .service(nodes_watch)
            .service(consensus)
    })
    .bind(addr)
    .expect("Check bind path!")
    .run()
    .await
}

#[get("/v1/")]
async fn nodes_watch(
    _some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    _some_node: web::Data<Arc<Mutex<Node>>>,
) -> Result<HttpResponse, Error> {
    println!("Start work! {}{}",_some_node.lock().unwrap().addr.clone(), _some_node.lock().unwrap().port.clone());
    let http = String::from("http://");

    let mut rng = rand::thread_rng();
    let rand_num: u64 = rng.gen();
    let t_block_mint = Duration::new(rand_num % 3 + 1, 0);
    println!("{:?}", t_block_mint);
    if _some_blockch
        .lock()
        .expect("Can`t check genesis")
        .blocks
        .len()
        == 0
    {
        for node in &_some_node
            .lock()
            .expect("Can`t send mint block!")
            .config_list
        {
            match Client::new()
                .get(http.clone() + node + "/v1/private/genesis/")
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
    loop {
        async_std::task::sleep(t_block_mint).await;
        _some_blockch.lock().expect("Can`t mint!").mint();
        println!("{:?}", _some_blockch.lock().unwrap());
        for node in &_some_node
            .lock()
            .expect("Can`t send mint block!")
            .config_list
        {
            let url = http.clone() + &node + "/v1/private/blocks/new";
            let block = _some_blockch
                .lock()
                .expect("Can`t get mint block!")
                .blocks
                .back()
                .expect("Can`t get mint block! Block does not exist!")
                .clone();
            actix_web::rt::spawn(async move {
                match Client::new().post(url).send_json(&block.clone()).await {
                    Ok(_) => println!("Sending was successful! {:?}", std::thread::current()),
                    Err(e) => println!("Sending wasn`t successful! Error:\n{:?}", e),
                };
            });
        }
    }

    //Ok(HttpResponse::Ok().json(format!("My watch has ended!",)))
}
#[get("/v1/start")]
async fn start_working(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    let mut config: HashSet<String> = some_node.lock().expect("Can`t start!").config_list.clone();
    // config.insert(
    //     some_node.lock().expect("Can`t start!").addr.clone() 
    //     + &some_node.lock().expect("Can`t start!").port
    // );
    for node in config {
        let http = String::from("http://");
        let url = http + &node + "/v1/";
        let client = Client::new();
        println!("{:?}", std::thread::current());
        match client.get(url.clone()).send().await {
            Ok(_) => println!("{} starts to work!", &node),
            Err(e) => println!("Start error! URL {}\n{} will not work! Error:\n{:?}",url, &node, e),
        };
    }
    Client::new().get(String::from("http://") + &some_node.lock().expect("Can`t start!").addr.clone() + &some_node.lock().expect("Can`t start!").port +  "/v1/").send().await;
    Ok(HttpResponse::Ok().json(format!("Yes, my lord!")))
}
async fn add_block(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_forks: web::Data<Arc<Mutex<Vec<LinkedList<Block>>>>>,
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
    if some_forks.lock().expect("Can`t create fork!").len() == 0 {
        match some_blockch.lock().expect("Can`t add block!").blocks.back() {
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
                    let last_valid_block = some_blockch
                        .lock()
                        .expect("Can`t create fork!")
                        .blocks
                        .pop_back()
                        .expect("Can`t get add last block to fork chain!");
                    let mut fork: LinkedList<Block> = LinkedList::new();
                    fork.push_back(some_block);
                    some_forks
                        .lock()
                        .expect("Can`t push block in fork!")
                        .push(fork);
                    let mut fork: LinkedList<Block> = LinkedList::new();
                    fork.push_back(last_valid_block);
                    some_forks
                        .lock()
                        .expect("Can`t push last block in fork!")
                        .push(fork);
                    return Ok(HttpResponse::Ok().json(format!("There has been a network fork!")));
                } else {
                    //invalid block
                    println!("Wrong previous hash!");
                    return Err(error::ErrorBadRequest("The hashes don't match!"));
                }
            }
        }
    } else {
        //with forks
        for chain in 0..some_forks
            .lock()
            .expect("Can't add a some block to the fork")
            .len()
        {
            if some_forks
                .lock()
                .expect("Can't match a block")
                .get(chain)
                .expect("Can`t some get fork!")
                .back()
                .expect("Can`t get last block from fork!")
                .hash
                == some_block.previous_hash
            {
                some_forks
                    .lock()
                    .expect("Can't add a block to the fork")
                    .get_mut(chain)
                    .expect("Can`t get fork!")
                    .push_back(some_block);
                return Ok(HttpResponse::Ok().json(format!("Adding a block is successful!")));
            }
        }
    }
    Ok(HttpResponse::Ok().json(format!("Adding a block successfully!")))
}

#[get("/v1/private/consensus/")]
async fn consensus(
    some_forks: web::Data<Arc<Mutex<Vec<LinkedList<Block>>>>>,
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
) -> Result<HttpResponse, Error> {
    let chain = 3;
    if some_forks
        .lock()
        .expect("Can't add a block to the fork")
        .get(chain)
        .expect("Can`t get some fork!")
        .len()
        > 5
    {
        some_blockch
            .lock()
            .expect("Can`t append block!")
            .blocks
            .append(
                &mut some_forks
                    .lock()
                    .expect("Can't get a fork to append!")
                    .get(chain)
                    .expect("Can't get a block to append!")
                    .clone(),
            );
        some_forks.lock().expect("Can't clear forks!").clear();
    }

    Ok(HttpResponse::Ok().json(format!("Yes, my lord!")))
}

#[get("/v1/private/")]
async fn get_blockchain(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
) -> Result<HttpResponse, Error> {
    let some_blockch = some_blockch.lock().expect("Can`t get blockchain!");
    let some_blockchain = Blockchain {
        blocks: some_blockch.blocks.clone(),
        transactions_queue: some_blockch.transactions_queue.clone(),
    };
    Ok(HttpResponse::Ok().json(some_blockchain))
}

#[get("/v1/private/genesis/")]
async fn genesis_generate(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    some_node: web::Data<Arc<Mutex<Node>>>,
) -> Result<HttpResponse, Error> {
    //if genesis already exists
    if some_blockch
        .lock()
        .expect("Can`t create genesis!")
        .blocks
        .len()
        != 0
    {
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
                Err(_) => println!("Node {} is not deployed!", &node),
            };
        }
        Ok(HttpResponse::Ok().json(format!(
            "Genesis block created successfully! {:?}",
            genesis_block
        )))
    }
}

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
                some_blockch.transactions_queue = blockchain.transactions_queue.clone();
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
    let some_node = some_node.lock().expect("Can`t show status!");
    Ok(HttpResponse::Ok().json(some_node.config_list.clone()))
}
#[get("/v1/public/node/status")]
async fn node_status(some_node: web::Data<Arc<Mutex<Node>>>) -> Result<HttpResponse, Error> {
    let some_node = some_node.lock().expect("Can`t show status!");
    Ok(HttpResponse::Ok().json(format!("{:?}", some_node)))
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
        "Top block is: {:#?} len {} queue {:#?} len {}",
        some_blockchain.blocks.back(),
        some_blockchain.blocks.len(),
        some_blockchain.transactions_queue,
        some_blockchain.transactions_queue.len()
    )))
}

async fn add_transactions(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
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
    let some_tr = serde_json::from_slice::<Transaction>(&body)?;
    let mut some_blockch = some_blockch.lock().expect("Can`t add transaction!");
    some_blockch.new_transaction(some_tr.from.clone(), some_tr.to.clone(), some_tr.amount);
    Ok(HttpResponse::Ok().json(some_tr))
}

#[get("/v1/public/transactions/:{index}")]
async fn view_trans(
    some_blockch: web::Data<Arc<Mutex<Blockchain>>>,
    web::Path(index): web::Path<usize>,
) -> Result<HttpResponse, Error> {
    let some_blockchain = some_blockch.lock().expect("Can`t get transaction!");
    Ok(HttpResponse::Ok().json(format!(
        "Transaction number {} is: {:#?} len {}",
        index,
        some_blockchain.transactions_queue.get(index),
        some_blockchain.transactions_queue.len()
    )))
}

#[get("/v1/public/blocks/:{index}")]
async fn view_by_index(
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
