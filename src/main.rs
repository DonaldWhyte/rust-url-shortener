#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

mod constants;
mod service;
mod token;

use log::LogLevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::config::{Appender, Config, Root};

fn init_logs() {
    let stdout = ConsoleAppender::builder().build();
    let file = RollingFileAppender::builder()
        .build("./url_shortener.log", Box::new(CompoundPolicy::new(
            Box::new(SizeTrigger::new(50 * 1024 * 1024)), // 50 MB
            Box::new(FixedWindowRoller::builder()
                .build("url_shortener.{}.log", 10)
                .unwrap()))))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(Root::builder().appenders(vec!("stdout", "file"))
            .build(LogLevelFilter::Info))
        .unwrap();
    let _ = log4rs::init_config(config);
}

fn main() {
    init_logs();

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

    info!("Creating Redis connection pool");
    let connection_str =
        "redis://".to_string() + redis_host + ":" + &redis_port.to_string();
    let manager = r2d2_redis::RedisConnectionManager::new(
        connection_str.as_ref()).unwrap();
    let config = Default::default();
    let connection_pool = r2d2::Pool::new(config, manager).unwrap();

    service::start_service(connection_pool, address, port);
}
