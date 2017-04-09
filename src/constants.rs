extern crate iron;

use std::result::Result::{Ok};
use self::iron::IronResult;
use self::iron::response::Response;
use self::iron::status::Status;

pub const SERVICE_DEFAULT_ADDRESS : &str = "localhost";
pub const REDIS_DEFAULT_HOST : &str = "localhost";
pub const REDIS_DEFAULT_PORT : u16 = 6379;

pub const URL_BASENAME: &str = "localhost";

pub fn internal_service_error() -> IronResult<Response> {
    Ok(Response::with((Status::InternalServerError, "internal service error")))
}
