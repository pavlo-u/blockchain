extern crate openssl;

use openssl::{rsa::Rsa, symm::Cipher};
use std::process;

pub fn key_gen(passphrase: String) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let rsa = Rsa::generate(2048).unwrap_or_else(|err| {
        println!("Key generation problem: {}", err);
        process::exit(1);
    });
    let pkey = rsa
        .private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes())
        .unwrap_or_else(|err| {
            println!("Public key generation problem: {}", err);
            process::exit(1);
        });

    let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap_or_else(|err| {
        println!("Private key generation problem: {}", err);
        process::exit(1);
    });
    (Some(pkey), Some(pub_key))
    //println!("{:?}", std::str::from_utf8(pub_key.as_slice()))
    // (, std::str::from_utf8(pub_key.as_slice()).unwrap_or_else(|err| {
    //     println!("Key generation problem: {}", err);
    //     process::exit(1);
    // }))
}
