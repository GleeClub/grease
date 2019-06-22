extern crate grease_api;

use grease_api::routes::handle_request;

fn main() {
    dotenv::dotenv().ok();
    cgi::handle(handle_request);
}
