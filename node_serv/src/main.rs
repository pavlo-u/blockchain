#[path = "../blockchain/node.rs"]
pub mod node;

fn main() {
    node::node_main().expect("Can`t deploy!");
}
