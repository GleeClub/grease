use thiserror::Error;
use time::IndeterminateOffsetError;

pub type GreaseResult<T> = Result<T, GreaseError>;

#[derive(Error, Debug)]
pub enum GreaseError {
    #[error("An error was returned from the database: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Failed to get the local time: {0}")]
    GetLocalTimeError(#[from] IndeterminateOffsetError),
    #[error("DATABASE_URL environment variable not provided")]
    DbUrlNotProvided,
    #[error("Error arose from GraphQL API: {0}")]
    GqlError(String),
}
