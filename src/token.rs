extern crate rustc_serialize;
extern crate sha2;

use self::rustc_serialize::hex::ToHex;
use self::sha2::{Digest, Sha256};
use constants;

pub fn to_token(url: &str) -> String {
    let mut hash = Sha256::default();
    hash.input(url.as_bytes());
    hash.result().as_slice().to_hex()
}

pub fn token_to_url(token: &str) -> String {
    // TODO: make basename configurable
    format!("http://{}:3000/{}", constants::URL_BASENAME, token)
}
