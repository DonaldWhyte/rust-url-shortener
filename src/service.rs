extern crate iron;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate router;

use std::result::Result;
use self::iron::IronResult;
use self::iron::prelude::*;
use self::iron::status::*;
use self::iron::Url;
use self::persistent::Read;
use self::r2d2::Pool;
use self::r2d2_redis::RedisConnectionManager;


type RedisPool = Pool<RedisConnectionManager>;
struct Redis;
impl iron::typemap::Key for Redis {
    type Value = RedisPool;
}

macro_rules! validate_url {
    ($url:expr) => {
        match iron::Url::parse($url) {
            Ok(_) => { },
            Err(_) => {
                Ok(Response::with((Status::BadRequest, "Malformed URL")));
            }
        }
    }
}

macro_rules! return_if_error {
    ($expr:expr) => {
        match $expr {
            Ok(_) => { },
            Err(e) => { return e }
        }
    }
}

fn resolve_or_shorten_url(connection: &i32, url: &str) -> Result<String, String> {
    match connection.get(url) {
        Ok(token) => { Ok(token) }  // TODO: to full URL
        Err(_) => { create_shortened_url(connection, url) }
    }
}

fn create_shortened_url(connection: &i32, url: &str) -> Result<String, String> {
    let token = "TODO";
    return_if_error!(connection.set(token, url));
    return_if_error!(connection.set(url, token));
    let shortened_url = token; // TODO
    Ok(shortened_url)
}

fn shorten_handler(req: &mut Request) -> IronResult<Response> {
    match req.url.clone().query() {
        None => { Ok(Response::with((Status::BadRequest, "URL missing in query"))) },
        Some(s) => {
            let (arg_name, arg_val) = s.split_at(4);
            if arg_name == "url=" {
                validate_url!(arg_val);
                let pool = req.get::<Read<Redis>>().unwrap().clone();
                let connection = pool.get().unwrap();
                match resolve_or_shorten_url(&connection, arg_val) {
                    Ok(token) => {
                        Status::Created,
                    }
                    Err(e) => {
                        Ok(Response::with((Status::TODO, e)))
                    }
                }
            } else {
                Ok(Response::with((Status::BadRequest, "Malformed query string")))
            }
        }
    }
}

/*fn resolve_handler(req: &mut Request) -> IronResult<Response> {
    // TODO
}*/

pub fn start_service(connection_pool: RedisPool, address: &str, port: u16) {
    let mut router = router::Router::new();
    router.get("/shorten", shorten_handler, "shorten");
    //router.get("/:token", resolve_handler, "resolver");

    let mut chain = Chain::new(router);
    chain.link_before(Read::<Redis>::one(connection_pool));
    let binding_str = address.to_string() + ":" + &port.to_string();
    Iron::new(chain).http(binding_str).unwrap();
}