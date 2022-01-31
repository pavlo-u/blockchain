#[path = "../blockchain/node.rs"]
pub mod node;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    node::blockchain_network(args.pop().expect("Blockchain network error")).expect("why");
}
