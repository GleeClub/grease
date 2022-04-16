use anyhow::{Context as _, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptySubscription, Guard, Request, Schema, Variables};
use cgi::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use cgi::http::response::Builder;
use cgi::http::Method;
use serde::Deserialize;

use crate::db::DbConn;
use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::models::member::Member;
use crate::util::{get_token_from_header, gql_err_to_anyhow};

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
    if request.method() == Method::GET {
        return Ok(cgi::html_response(
            200,
            playground_source(GraphQLPlaygroundConfig::new("/")),
        ));
    }

    // TODO: handle commit
    let conn = DbConn::connect().await?;
    let user = load_user_from_request(&request, &conn).await?;

    let body: RequestBody =
        serde_json::from_slice(request.body()).context("Invalid request body")?;
    let request = Request::new(body.query)
        .variables(body.variables)
        .data(conn);
    let request = if let Some(user) = user {
        request.data(user)
    } else {
        request
    };

    let schema = Schema::new(QueryRoot, MutationRoot, EmptySubscription);
    let response = schema.execute(request).await;
    let body = serde_json::to_vec(&response).context("Failed to serialize response")?;

    Builder::new()
        .status(200)
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(CONTENT_LENGTH, body.len().to_string().as_str())
        .body(body)
        .context("Failed to build response")
}

async fn load_user_from_request(request: &cgi::Request, conn: &DbConn) -> Result<Option<Member>> {
    if let Some(token) = get_token_from_header(request) {
        let member = Member::with_token(token, conn)
            .await
            .map_err(gql_err_to_anyhow)?;

        Ok(Some(member))
    } else {
        Ok(None)
    }
}
