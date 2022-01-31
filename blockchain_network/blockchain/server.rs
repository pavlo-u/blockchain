#[path = "../blockchain/key.rs"]
pub mod key;

use std::collections::HashSet;

use key::key_gen;
#[derive(Clone)]
pub struct Node {
    pkey: Vec<u8>,
    pub id: Vec<u8>,
    pub addr: String,
    pub port: String,
    pub config_list: HashSet<String>,
}
impl Node {
    pub fn new(addr: String, port: String) -> Node {
        let key_pair: (Option<Vec<u8>>, Option<Vec<u8>>) = key_gen(port.clone());
        let mut config = Node::load_config().expect("Can`t load config");
        config.remove(&(addr.clone() + &port));
        Node {
            pkey: key_pair.0.expect("Private key error"),
            id: key_pair.1.expect("Public key error"),
            addr,
            port,
            config_list: config,
        }
    }
    fn load_config() -> Result<HashSet<String>, serde_json::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open("config.json").expect("Can`t open file!");
        let mut some_buf = String::new();
        file.read_to_string(&mut some_buf)
            .expect("Can`t read file!");
        serde_json::from_str(&some_buf)
    }
}
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let byte_pub_key = self.id.clone();
        let pub_key = std::str::from_utf8(byte_pub_key.as_slice()).unwrap();
        write!(
            f,
            "Id: {}\naddr&port {}{}\nConfig list: {:?}",
            pub_key,
            &self.addr,
            &self.port,
            &self.config_list // "addr&port {}{}\nConfig list: {:?}",
                              // pub_key, &self.addr, &self.port, &self.config_list
        )
    }
}
