use anyhow::{Context as _, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptySubscription, Request, Schema, Variables};
use cgi::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use cgi::http::response::Builder;
use cgi::http::Method;
use serde::Deserialize;

use crate::db::DbConn;
use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;
use crate::models::member::Member;
use crate::util::{get_token_from_header, gql_err_to_anyhow};

pub mod guards;
pub mod mutation;
pub mod query;

pub const SUCCESS_MESSAGE: &str = "success";

#[derive(Deserialize)]
struct RequestBody {
    pub query: String,
    pub variables: Variables,
}

pub async fn handle(request: cgi::Request) -> Result<cgi::Response> {
    if request.method() == Method::GET {
        return Ok(cgi::html_response(
            200,
            playground_source(GraphQLPlaygroundConfig::new("/cgi-bin/grease")),
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

    let response = build_schema().execute(request).await;
    let body = serde_json::to_vec(&response).context("Failed to serialize response")?;

    Builder::new()
        .status(200)
        .header(CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(CONTENT_LENGTH, body.len().to_string().as_str())
        .body(body)
        .context("Failed to build response")
}

pub fn build_schema() -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription)
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
