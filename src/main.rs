#![feature(custom_attribute)]
#![feature(const_generics)]

extern crate serde;
extern crate serde_json;
extern crate diesel_derive_enum;
extern crate dotenv;
extern crate chrono;
extern crate app_route;
extern crate cgi;
#[macro_use]
extern crate diesel;
extern crate http;

mod auth;
mod db;
mod error;
mod routes;
mod extract;

use http::{response, header::{CONTENT_LENGTH, CONTENT_TYPE}};
use crate::routes::handle_request;

fn main() {
    cgi::handle(|request: cgi::Request| -> cgi::Response {
        let uri = request
            .headers()
            .get("x-cgi-path-info")
            .map(|uri| uri.to_str().unwrap())
            .unwrap_or("")
            .to_string();

        match handle_request(request, uri) {
            Ok(value) => {
                let body = value.to_string().into_bytes();
                response::Builder::new()
                    .status(200)
                    .header(CONTENT_TYPE, "application/json")
                    .header(CONTENT_LENGTH, body.len().to_string().as_str())
                    .body(body)
                    .unwrap()
            }
            Err(error) => error.as_response(),
        }
    });
}
