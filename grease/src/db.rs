use anyhow::{Result, Context as _};
use async_graphql::Context;
use sqlx::{Connection, MySql, MySqlConnection, Transaction};
use std::sync::{Mutex, Arc};

#[derive(Clone)]
pub struct DbConn<'c>(Arc<Mutex<Transaction<'c, MySql>>>);

pub async fn connect() -> Result<MySqlConnection> {
    let url = std::env::var("DATABASE_URL").context("Failed to get database URL")?;
    MySqlConnection::connect(&url).await.context("Faied to connect to DB")
}

pub fn get_conn<'c>(ctx: &'c Context<'_>) -> DbConn<'c> {
    ctx.data_unchecked::<Transaction<'c, MySql>>()
}
