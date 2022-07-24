use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type GreaseResult<T> = Result<T, GreaseError>;

#[derive(Error, Debug)]
pub enum GreaseError {
    #[error("An error was returned from the database: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Error arose from GraphQL API: {0}")]
    GqlError(String),
    #[error("Invalid token header: {0}")]
    InvalidTokenHeader(ToStrError),
}

impl IntoResponse for GreaseError {
    fn into_response(self) -> Response {
        let status = match &self {
            &GreaseError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            &GreaseError::GqlError(_) | &GreaseError::InvalidTokenHeader(_) => {
                StatusCode::BAD_REQUEST
            }
        };

        (status, self.to_string()).into_response()
    }
}
