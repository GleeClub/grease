use anyhow::{Context as _, Result};
use async_graphql::{Context, EmptySubscription, Guard, Request, Schema, Variables};
use cgi::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use serde::Deserialize;

use crate::db::DbConn;
use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::models::member::Member;

pub mod mutation;
pub mod permission;
pub mod query;

pub const SUCCESS_MESSAGE: &'static str = "success";

pub struct LoggedIn;

#[async_trait::async_trait]
impl Guard for LoggedIn {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
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
    pub variables: Variables,
}

pub async fn handle(request: cgi::Request) -> Result<cgi::Response> {
    let conn = DbConn::connect().await?;

    let body: RequestBody =
        serde_json::from_slice(request.body()).context("Invalid request body")?;
    let request = Request::new(body.query)
        .variables(body.variables)
        .data(conn);

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let response = schema.execute(request).await;
    let body = serde_json::to_vec(&response).context("Failed to serialize response")?;

    cgi::http::response::Builder::new()
        .status(200)
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(CONTENT_LENGTH, body.len().to_string().as_str())
        .body(body)
        .context("Failed to build response")
}
