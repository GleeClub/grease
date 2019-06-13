#![feature(custom_attribute)]
#![feature(drain_filter)]
#![recursion_limit = "128"]

extern crate app_route;
extern crate cgi;
extern crate chrono;
extern crate diesel_derive_enum;
extern crate dotenv;
extern crate extract_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate diesel;
extern crate http;
extern crate lettre;
extern crate lettre_email;

pub mod auth;
pub mod db;
pub mod error;
pub mod extract;
pub mod routes;
pub mod util;

use crate::routes::handle_request;

fn main() {
    cgi::handle(handle_request);
}
