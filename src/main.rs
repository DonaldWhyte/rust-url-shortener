#[macro_use]
extern crate clap;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Donald Whyte <don@donsoft.io>")
        (about: "Rust URL shortener backed by Redis")
        (@arg port: --port +takes_value +required "Port web service binds to")
        (@arg address: --address +takes_value "Address web service binds to [default: localhost]")
        (@arg redis_host: --redis_host +takes_value "Hostname backing Redis store runs on [default: localhost]")
        (@arg redis_port: --redis_port +takes_value "Port backing Redis store is listening on [default: 6379]")
    ).get_matches();
    let port = value_t!(matches.value_of("port"), u16).unwrap();
    let address = matches.value_of("address").unwrap_or("localhost");
    let redis_host = matches.value_of("redis_host").unwrap_or("localhost");
    let redis_port = value_t!(matches.value_of("redis_port"), u16).unwrap_or(6379);

    // TODO
}