//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website

fn main() {
    dotenv::dotenv().ok();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    cgi::handle(|request| {
        rt.block_on(async move { cgi::err_to_500(grease::graphql::handle(request).await) })
    });
}
