#[macro_use]
extern crate clap;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

mod constants;
mod service;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Donald Whyte <don@donsoft.io>")
        (about: "Rust URL shortener backed by Redis")
        (@arg port: --port +takes_value +required
            "Port web service binds to")
        (@arg address: --address +takes_value
            "Address web service binds to [default: localhost]")
        (@arg redis_host: --redis_host +takes_value
            "Hostname backing Redis store runs on [default: localhost]")
        (@arg redis_port: --redis_port +takes_value
            "Port backing Redis store is listening on [default: 6379]")
    ).get_matches();
    let port = value_t!(matches.value_of("port"), u16).unwrap();
    let address = matches.value_of("address").unwrap_or(
        constants::SERVICE_DEFAULT_ADDRESS);
    let redis_host = matches.value_of("redis_host").unwrap_or(
        constants::REDIS_DEFAULT_HOST);
    let redis_port = value_t!(matches.value_of("redis_port"), u16).unwrap_or(
        constants::REDIS_DEFAULT_PORT);

    let connection_str =
        "redis://".to_string() + redis_host + ":" + &redis_port.to_string();
    let manager = r2d2_redis::RedisConnectionManager::new(
        connection_str.as_ref()).unwrap();
    let config = Default::default();
    let connection_pool = r2d2::Pool::new(config, manager).unwrap();

    service::start_service(connection_pool, address, port);
}
