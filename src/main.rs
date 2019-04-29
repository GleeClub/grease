extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate warp;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel_derive_enum;
extern crate r2d2_diesel_mysql;
#[macro_use]
extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
extern crate chrono;

use warp::Filter;

mod auth;
mod db;
mod error;
mod models;
mod routes;

fn main() {
    let json_api = path!("grease" / "api").and(members::api().or(gallery::api()));
    // layout is:
    //   GET   /grease/api/members/ -> returns the current members
    //   POST  /grease/api/members/ -> adds a JSON-formatted member to the list
    //   GET   /grease/api/gallery/ -> returns the current gallery "images"
    //   POST  /grease/api/gallery/ -> adds a JSON-formatted "image" to the list
    warp::serve(json_api).run(([127, 0, 0, 1], 3030));
}
