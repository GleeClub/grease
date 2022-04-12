//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website.

#![feature(drain_filter, path_try_exists, once_cell)]

mod cron;
mod util;
mod db;
mod email;
mod file;
mod graphql;
mod models;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    if std::env::var("REQUEST_METHOD").is_ok() {
        cgi::handle(|request| rt.block_on(graphql::handle(request)));
        Ok(())
    } else {
        rt.block_on(async { cron::send_event_emails(None).await })
    }
}
