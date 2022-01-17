pub mod blockchain;
pub mod transaction;
use actix_web::{error, Error};
use actix_web::{get, web, HttpResponse, Responder};
use blockchain::Blockchain;
use futures::StreamExt;
use std::sync::Mutex;
use transaction::Transaction;

async fn add_transactions(
    some_blockch: web::Data<Mutex<Blockchain>>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > 262_144 {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    // body is loaded, now we can deserialize serde-json
    let some_tr = serde_json::from_slice::<Transaction>(&body)?;
    let mut some_blockch = some_blockch.lock().expect("Can`t add transaction!");
    some_blockch.new_transaction(some_tr.from.clone(), some_tr.to.clone(), some_tr.amount);
    Ok(HttpResponse::Ok().json(some_tr)) // <- send response
}
#[get("/v1/public/transactions/:{index}")]
async fn get_trans(
    some_blockch: web::Data<Mutex<Blockchain>>,
    web::Path(index): web::Path<usize>,
) -> impl Responder {
    let some_blockchain = some_blockch.lock().expect("Can`t get transaction!");
    format!(
        "Transaction number {} is:\n{:#?}\nlen {}",
        index,
        some_blockchain.transactions_queue.get(index),
        some_blockchain.transactions_queue.len()
    )
}
#[get("/v1/public/blocks/:{index}")]
async fn get_by_index(
    some_blockch: web::Data<Mutex<Blockchain>>,
    web::Path(index): web::Path<usize>,
) -> impl Responder {
    let some_blockchain = some_blockch.lock().expect("Can`t get block!");
    format!(
        "Block by number {} is:\n{:#?}\nlen {}",
        index,
        some_blockchain.blocks.iter().nth(index),
        some_blockchain.blocks.len()
    )
}
#[get("/v1/public/blocks/head")]
async fn get_top(some_blockch: web::Data<Mutex<Blockchain>>) -> impl Responder {
    let some_blockchain = some_blockch.lock().expect("Can`t get head!");
    format!(
        "Top block is:\n{:#?}\nlen {}\nqueue {:#?}\nlen {}",
        some_blockchain.blocks.back(),
        some_blockchain.blocks.len(),
        some_blockchain.transactions_queue,
        some_blockchain.transactions_queue.len()
    )
}

#[actix_web::main]
pub async fn node_main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let blockchain = Blockchain::new();
    let blnch = web::Data::new(Mutex::new(blockchain));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&blnch))
            .service(
                web::resource("v1/public/transactions/new").route(web::post().to(add_transactions)),
            )
            .service(get_trans)
            .service(get_by_index)
            .service(get_top)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
