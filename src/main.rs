//! The backend for the Georgia Tech Glee Club's website

#![feature(drain_filter, path_try_exists, once_cell, generic_associated_types)]

mod email;
mod error;
mod file;
mod graphql;
mod models;
mod util;

use std::net::SocketAddr;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Request, Response};
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::MySqlPool;
use time::{Duration, OffsetDateTime};

use crate::error::{GreaseError, GreaseResult};
use crate::graphql::build_schema;
use crate::models::event::Event;
use crate::models::member::Member;
use crate::models::semester::Semester;
use crate::util::{connect_to_db, now};

const GREASE_TOKEN: &'static str = "GREASE_TOKEN";
const API_URL: &'static str = "api.glubhub.org";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(playground).post(query))
        .route("/send-emails", get(send_emails));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn query(Json(request): Json<Request>, headers: HeaderMap) -> GreaseResult<Json<Response>> {
    let pool = connect_to_db().await?;
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

async fn playground(headers: HeaderMap) -> GreaseResult<String> {
    let mut config = GraphQLPlaygroundConfig::new(API_URL);
    if let Some(header) = get_token(&headers)? {
        config = config.with_header(GREASE_TOKEN, header);
    }

    Ok(playground_source(config))
}

// TODO: make it so this can only be called systematically
async fn send_emails() -> GreaseResult<StatusCode> {
    // TODO: handle commit
    let pool = connect_to_db().await?;
    let since = now()? - Duration::hours(1);

    for _event in events_to_notify_about(&pool, since).await {
        // let email = email_for_event(&event, &pool).await?;
        // email.send().await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn events_to_notify_about(
    pool: &MySqlPool,
    since: OffsetDateTime,
) -> async_graphql::Result<impl Iterator<Item = Event>> {
    let current_semester = Semester::get_current(&pool).await?;
    let all_events = Event::for_semester(&current_semester.name, &pool).await?;

    let two_days_from_now = now()? + Duration::days(2);
    let two_days_from_last_checked = since + Duration::days(2);

    Ok(all_events.into_iter().filter(move |event| {
        event.call_time.0 < two_days_from_now && event.call_time.0 > two_days_from_last_checked
    }))
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
