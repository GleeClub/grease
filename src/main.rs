#![feature(custom_attribute)]
#![feature(drain_filter)]
#![feature(box_syntax)]
#![feature(const_fn)]
#![recursion_limit = "128"]

extern crate base64;
extern crate cgi;
extern crate chrono;
extern crate dotenv;
extern crate glob;
extern crate grease_derive;
extern crate http;
extern crate mysql;
extern crate mysql_enum;
extern crate pinto;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
extern crate url;
// extern crate lettre;
// extern crate lettre_email;
#[cfg(test)]
extern crate mocktopus;

mod auth;
mod db;
mod error;
mod extract;
pub mod routes;
mod util;

use routes::handle_request;

fn main() {
    dotenv::dotenv().ok();
    cgi::handle(handle_request);
}
