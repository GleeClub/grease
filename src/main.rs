//! The backend for the Georgia Tech Glee Club's website

#![feature(drain_filter, fs_try_exists, once_cell, generic_associated_types)]

mod email;
mod error;
mod file;
mod graphql;
mod models;
mod util;

use std::env::var;
use std::net::SocketAddr;

use anyhow::Context;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Request, Response as GraphQLResponse};
use axum::extract::Query;
use axum::headers::{ContentType, HeaderMap};
use axum::routing::get;
use axum::{Extension, Json, Router, TypedHeader};
use serde::Deserialize;
use sqlx::PgPool;

use crate::email::run_email_loop;
use crate::error::{GreaseError, GreaseResult};
use crate::graphql::build_schema;
use crate::models::member::Member;

const GREASE_TOKEN: &'static str = "GREASE_TOKEN";
const API_URL: &'static str = "https://api.glubhub.org";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let db_uri = var("DATABASE_URL").context("DATABASE_URL not set")?;
    let pool = PgPool::connect(&db_uri)
        .await
        .context("Failed to connect to database")?;

    // Only run the email sending loop if an interval time is set
    if let Ok(email_send_interval_seconds) = var("EMAIL_SEND_INTERVAL_SECONDS") {
        let email_send_interval_seconds = email_send_interval_seconds
            .parse()
            .context("EMAIL_SEND_INTERVAL_SECONDS must be an integer")?;
        tokio::spawn(run_email_loop(email_send_interval_seconds, pool.clone()));
    }

    let app = Router::new()
        .route("/", get(playground).post(query))
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn query(
    Json(request): Json<Request>,
    headers: HeaderMap,
    Extension(pool): Extension<PgPool>,
) -> GreaseResult<Json<GraphQLResponse>> {
    let user = if let Some(token) = get_token(&headers)? {
        Some(
            Member::with_token(token, &pool)
                .await
                .map_err(|err| GreaseError::GqlError(err.message))?,
        )
    } else {
        None
    };

    let request = Request::new(request.query)
        .variables(request.variables)
        .data(pool);
    let request = if let Some(user) = user {
        request.data(user)
    } else {
        request
    };

    Ok(Json(build_schema().execute(request).await))
}

#[derive(Deserialize)]
struct OptionalToken {
    #[serde(default)]
    pub token: Option<String>,
}

async fn playground(
    headers: HeaderMap,
    params: Query<OptionalToken>,
) -> GreaseResult<(TypedHeader<ContentType>, String)> {
    let mut config = GraphQLPlaygroundConfig::new(API_URL);
    if let Some(header) = get_token(&headers)?.or(params.token.as_deref()) {
        config = config.with_header(GREASE_TOKEN, header);
    }

    Ok((TypedHeader(ContentType::html()), playground_source(config)))
}

fn get_token(headers: &HeaderMap) -> GreaseResult<Option<&str>> {
    headers
        .iter()
        .find_map(|(name, value)| {
            if name == GREASE_TOKEN {
                Some(value.to_str().map_err(GreaseError::InvalidTokenHeader))
            } else {
                None
            }
        })
        .transpose()
}
