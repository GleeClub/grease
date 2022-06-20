//! The backend for the Georgia Tech Glee Club's website

use async_graphql::{Request, Response, Variables};
use grease::error::{GreaseError, GreaseResult};
use grease::graphql::build_schema;
use grease::models::member::Member;
use grease::util::connect_to_db;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct GraphQLRequest {
    #[serde(default)]
    pub token: Option<String>,
    pub query: String,
    pub variables: Variables,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = service_fn(handle);
    lambda_runtime::run(handler).await
}

async fn handle(request: LambdaEvent<GraphQLRequest>) -> GreaseResult<Response> {
    let pool = connect_to_db().await?;
    let user = if let Some(token) = request.payload.token {
        Some(
            Member::with_token(&token, &pool)
                .await
                .map_err(|err| GreaseError::GqlError(err.message))?,
        )
    } else {
        None
    };

    let request = Request::new(request.payload.query)
        .variables(request.payload.variables)
        .data(pool);
    let request = if let Some(user) = user {
        request.data(user)
    } else {
        request
    };

    Ok(build_schema().execute(request).await)
}
