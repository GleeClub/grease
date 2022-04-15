//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website

use grease::{cron, graphql};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    if std::env::var("REQUEST_METHOD").is_ok() {
        cgi::handle(|request| {
            rt.block_on(async move { cgi::err_to_500(graphql::handle(request).await) })
        });

        Ok(())
    } else {
        rt.block_on(async { cron::send_event_emails(None).await })
    }
}
