extern crate iron;
extern crate logger;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate router;

use std::error::Error;
use std::result::Result;
use std::result::Result::{Ok, Err};
use self::iron::IronResult;
use self::iron::modifiers::Redirect;
use self::iron::prelude::*;
use self::iron::status::*;
use self::iron::Url;
use self::logger::Format;
use self::logger::Logger;
use self::persistent::Read;
use self::r2d2::Pool;
use self::r2d2_redis::RedisConnectionManager;
use self::redis::{Commands, RedisResult};
use token;

// -----------------------------------------------------------------------------
// * Redis
// -----------------------------------------------------------------------------
type RedisPool = Pool<RedisConnectionManager>;
struct Redis;
impl iron::typemap::Key for Redis {
    type Value = RedisPool;
}

fn shortern_url(connection: &redis::Connection, url: &str) -> Result<String, String> {
    let token = token::to_token(url);
    let result: RedisResult<()> = connection.set(&token, url);
    match result {
        Ok(_) => Ok(token::token_to_url(&token)),
        Err(e) => Err(e.description().to_owned())
    }
}

fn resolve_token(connection: &redis::Connection, token: &str) -> Result<Option<String>, String> {
    let result: RedisResult<Option<String>> = connection.get(token);
    match result {
        Ok(url) => Ok(url),
        Err(e) => {
            let missing = false;
            if missing {
                Ok(None)
            } else {
                Err(e.description().to_owned())
            }
        }
    }
}

// -----------------------------------------------------------------------------
// * Service
// -----------------------------------------------------------------------------
fn internal_service_error(error_message: &str) -> IronResult<Response> {
    Ok(Response::with((
        Status::InternalServerError,
        format!("internal service error: {}", error_message)
    )))
}

fn shorten_handler(req: &mut Request) -> IronResult<Response> {
    match req.url.clone().query() {
        None => {
            Ok(Response::with((Status::BadRequest, "URL missing in query")))
        },
        Some(s) => {
            let (arg_name, arg_val) = s.split_at(4);
            if arg_name == "url=" {
                if let Err(_) = Url::parse(arg_val) {
                    return Ok(Response::with((Status::BadRequest, "Malformed URL")));
                }

                // Create token and return full shortened URL to client
                let connection_pool = req.get::<Read<Redis>>().unwrap().clone();
                let ref connection = connection_pool.get().unwrap();
                match shortern_url(&connection, arg_val) {
                    Ok(shortened_url) => {
                        Ok(Response::with((Status::Ok, shortened_url)))
                    },
                    Err(e) => {
                        internal_service_error(&e)
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
    let ref connection = connection_pool.get().unwrap();
    let token = req.url.path()[0];
    info!("Resolving token {:?}", token);
    match resolve_token(&connection, token) {
        Ok(resolved_url) => {
            match resolved_url {
                Some(url) => {
                    match Url::parse(&url) {
                        Ok(parsed_url) => {
                            Ok(Response::with((
                                Status::MovedPermanently, Redirect(parsed_url))))
                        },
                        Err(e) => {
                            internal_service_error(&e)
                        }
                    }
                },
                None => {
                    Ok(Response::with((Status::NotFound)))
                }
            }
        },
        Err(e) => {
            internal_service_error(&e)
        }
    }
}

pub fn start_service(connection_pool: RedisPool, address: &str, port: u16) {
    let mut router = router::Router::new();
    router.get("/shorten", shorten_handler, "shorten");
    router.get("/:token", resolve_handler, "resolver");
    let mut chain = Chain::new(router);

    let format = Format::new(
        "Uri: {uri}, Method: {method}, Status: {status}, Duration: {response-time}, Time: {request-time}");
    let (logger_before, logger_after) = Logger::new(Some(format.unwrap()));
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    info!("Creating Redis connection pool");
    chain.link_before(Read::<Redis>::one(connection_pool));

    let binding_str = address.to_string() + ":" + &port.to_string();
    info!("Starting service and binding to {:?}", binding_str);
    Iron::new(chain).http(binding_str).unwrap();
}
