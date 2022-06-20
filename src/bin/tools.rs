use std::env::args;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use grease::graphql::build_schema;

fn main() {
    match args().nth(1).as_deref() {
        Some("print-schema") => {
            println!("{}", build_schema().sdl());
        }
        Some("print-playground") => {
            println!(
                "{}",
                playground_source(GraphQLPlaygroundConfig::new("playground.glubhub.org")),
            );
        }
        Some(other) => {
            eprintln!("Unexpected command `{other}`. Please use either `print-schema` or `print-playground`.");
        }
        None => {
            eprintln!(
                "No command provided. Please use either `print-schema` or `print-playground`."
            );
        }
    }
}
