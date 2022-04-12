use anyhow::{Result, Context as _};
use async_graphql::Context;
use sqlx::{Connection, MySql, MySqlConnection, Transaction};

pub type DbConn<'c> = &'c mut Transaction<'c, MySql>;

pub async fn connect() -> Result<MySqlConnection> {
    let url = std::env::var("DATABASE_URL").context("Failed to get database URL")?;
    MySqlConnection::connect(&url).await.context("Faied to connect to DB")
}

pub fn get_conn<'c>(ctx: &'c Context<'_>) -> Transaction<'c, MySql> {
    ctx.data_unchecked::<Transaction<'c, MySql>>()
}
