use blockchain::Blockchain;
use blockchain::Backup;

fn main() {
    let mut blockch: Blockchain = Blockchain::new();
    let duration_sec: u64 = 8;
    blockch.fork_chain(duration_sec);
    blockch.save("State".to_string()).expect("Unsave!");
    let x = Blockchain::load("State".to_string()).unwrap();
    println!("{:#?}", x);
    
}
