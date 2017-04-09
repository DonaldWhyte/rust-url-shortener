extern crate iron;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate router;
extern crate rustc_serialize;
extern crate sha2;

use std::error::Error;
use std::result::Result;
use std::result::Result::{Ok, Err};
use self::iron::IronResult;
use self::iron::modifiers::Redirect;
use self::iron::prelude::*;
use self::iron::status::*;
use self::iron::Url;
use self::persistent::Read;
use self::r2d2::Pool;
use self::r2d2_redis::RedisConnectionManager;
use self::redis::{Commands, RedisResult};
use self::rustc_serialize::hex::ToHex;
use self::sha2::{Digest, Sha256};
use constants;


type RedisPool = Pool<RedisConnectionManager>;
struct Redis;
impl iron::typemap::Key for Redis {
    type Value = RedisPool;
}

fn token_to_url(token: &str) -> String {
    // TODO: make basename configurable
    format!("http://{}/{}", constants::URL_BASENAME, token)
}

fn resolve_or_shorten_url(connection: &redis::Connection, url: &str) -> Result<String, String> {
    let result: RedisResult<String> = connection.get(url);
    match result {
        Ok(token) => Ok(token_to_url(&token)),
        Err(_) => create_shortened_url(connection, url)
    }
}

fn create_token(url: &str) -> String {
    let mut hash = Sha256::default();
    hash.input(url.as_bytes());
    hash.result().as_slice().to_hex()
}

fn create_shortened_url(connection: &redis::Connection, url: &str) -> Result<String, String> {
    let token = create_token(url);

    // Need to assign RedisResults to variables first, since the compiler can't
    // deduce
    let result1: RedisResult<()> = connection.set(&token, url);
    match result1 {
        Ok(_) => {
            let result2: RedisResult<()> = connection.set(url, &token);
            match result2 {
                Ok(_) => Ok(token_to_url(&token)),
                Err(e) => Err(e.description().to_owned())
            }
        },
        Err(e) => Err(e.description().to_owned())
    }
}

fn shorten_handler(req: &mut Request) -> IronResult<Response> {
    match req.url.clone().query() {
        None => {
            Ok(Response::with((Status::BadRequest, "URL missing in query")))
        },
        Some(s) => {
            let (arg_name, arg_val) = s.split_at(4);
            if arg_name == "url=" {
                // Validat
                if let Err(_) = Url::parse(arg_val) {
                    // TODO: log error
                    return Ok(Response::with((Status::BadRequest, "Malformed URL")));
                }

                // Create token and return full shortened URL to client
                let connection_pool = req.get::<Read<Redis>>().unwrap().clone();
                let ref connection = connection_pool.get().unwrap();
                match resolve_or_shorten_url(&connection, arg_val) {
                    Ok(shortened_url) => {
                        Ok(Response::with((Status::Created, shortened_url)))
                    },
                    Err(e) => {
                        // TODO: log error here
                        constants::internal_service_error()
                    }
                }
            } else {
                Ok(Response::with((Status::BadRequest, "Malformed query string")))
            }
        }
    }
}

fn resolve_handler(req: &mut Request) -> IronResult<Response> {
    let connection_pool = req.get::<Read<Redis>>().unwrap().clone();
    let token = req.url.path()[0];

    let ref connection = connection_pool.get().unwrap();
    let result: RedisResult<String> = connection.get(token);
    match result {
        Ok(resolved_url) => {
            match Url::parse(&resolved_url) {
                Ok(parsed_url) => {
                    Ok(Response::with((
                        Status::MovedPermanently, Redirect(parsed_url))))
                },
                Err(e) => {
                    constants::internal_service_error()
                }
            }
        },
        Err(e) => {
            // TODO: log error here
            constants::internal_service_error()
        }
    }
}

pub fn start_service(connection_pool: RedisPool, address: &str, port: u16) {
    let mut router = router::Router::new();
    router.get("/shorten", shorten_handler, "shorten");
    router.get("/:token", resolve_handler, "resolver");

    let mut chain = Chain::new(router);
    chain.link_before(Read::<Redis>::one(connection_pool));
    let binding_str = address.to_string() + ":" + &port.to_string();
    Iron::new(chain).http(binding_str).unwrap();
}
