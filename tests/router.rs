extern crate cgi;
extern crate grease_api;
extern crate http;
extern crate serde_json;
extern crate speculate;

use grease_api::error::GreaseError;
use grease_api::routes::{handle, basic_success};
use http::request::Builder;
use serde_json::{json, Value};
use speculate::speculate;
use std::fmt::Display;

const MEMBER_EMAIL: &'static str = "joe.schmoe@gmail.com";
const MEMBER_PASS_HASH: &'static str = "hashedpassword";
const TOKEN: &'static str = "randomtoken123";

fn make_request<T: Display>(
    method: &str,
    path: &[&str],
    params: &[(&str, &str)],
    include_token: bool,
    body: T,
) -> cgi::Request {
    let body = body.to_string().into_bytes();
    let path = path
        .iter()
        .map(|segment| segment.to_string())
        .collect::<Vec<String>>()
        .join("/");
    let mut params = params
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<String>>()
        .join("&");
    if include_token {
        params = if params.len() == 0 {
            format!("token={}", TOKEN)
        } else {
            format!("token={}&{}", TOKEN, params)
        };
    }
    let url = format!("https://gleeclub.gatech.edu/{}?{}", path, params);

    Builder::new().uri(url).method(method).body(body).unwrap()
}

fn test_handle(request: cgi::Request) -> (u16, Value) {
    match handle(&request) {
        Ok(value) => (200, value),
        Err(error) => error.as_response(),
    }
}

fn test_get(path: &[&str], params: &[(&str, &str)]) -> (u16, Value) {
    let request = make_request("GET", path, params, true, "");
    test_handle(request)
}

fn test_post<T: Display>(path: &[&str], params: &[(&str, &str)], body: T) -> (u16, Value) {
    let request = make_request("POST", path, params, true, body);
    test_handle(request)
}

speculate! {
    describe "authorization" {
        before {
            let success = (200, basic_success());
        }

        it "can login" {
            assert_eq!(test_get(&["members", MEMBER_EMAIL], &[]), GreaseError::Unauthorized.as_response());

            let login_form = json!({
                "email": MEMBER_EMAIL,
                "pass_hash": MEMBER_PASS_HASH
            });
            assert_eq!(test_post(&["login"], &[], login_form), success);
        }
    }
}
