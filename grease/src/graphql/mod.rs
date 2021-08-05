use anyhow::Context;
use async_graphql::guard::Guard;
use async_graphql::types::EmptySubscription;
use async_graphql::{Context, Request, Result, Schema};
use sqlx::{Connection, MySql, MySqlConnection, Transaction};

use crate::db_conn::DbConn;
use crate::models::member::Member;

pub mod input;
pub mod mutation;
pub mod permission;
pub mod query;

pub struct LoggedIn;

#[async_trait::async_trait]
impl Guard for LoggedIn {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Member>().is_some() {
            Ok(())
        } else {
            Err("User must be logged in".into())
        }
    }
}

#[derive(Deserialize)]
struct RequestBody {
    pub query: String,
    pub variables: Value,
}

pub async fn handle(request: cgi::Request) -> anyhow::Result<cgi::Response> {
    let conn = DbConn::connect().await?;

    let body: RequestBody =
        serde_json::from_slice(request.body()).context("Invalid request body")?;
    let request = Request::new(body.query)
        .variables(body.variables)
        .data(transaction);

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let response = schema.execute(request);
    conn.finish().await?;

    let body = serde_json::to_vec(&value).context("Failed to serialize response")?;

    Ok(cgi::http::response::Builder::new()
        .status(status_code)
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(CONTENT_LENGTH, body.len().to_string().as_str())
        .body(body)
        .context("Failed to build response"))
}
