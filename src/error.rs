use db::models::Member;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum GreaseError {
    NotFound,
    AlreadyLoggedIn(String),
    Unauthorized,
    InvalidMethod,
    NotActiveYet(Member),
    Forbidden(Option<String>),
    ServerError(String),
    BadRequest(String),
    DbError(mysql::error::Error),
    FromRowError(mysql::FromRowError),
}

pub type GreaseResult<T> = Result<T, GreaseError>;

impl GreaseError {
    pub fn as_response(&self) -> (u16, Value) {
        match self {
            GreaseError::Unauthorized => (
                401,
                json!({
                    "message": "login required"
                }),
            ),
            GreaseError::InvalidMethod => (
                405,
                json!({
                    "message": "method not allowed"
                }),
            ),
            GreaseError::NotActiveYet(member) => (
                401,
                json!({
                    "message": "member not active yet",
                    "member": member.to_json()
                }),
            ),
            GreaseError::AlreadyLoggedIn(token) => (
                400,
                json!({
                    "message": "member already logged in",
                    "token": token
                }),
            ),
            GreaseError::Forbidden(Some(permission)) => (
                403,
                json!({
                    "message": "access forbidden",
                    "required_permission": permission
                }),
            ),
            GreaseError::Forbidden(None) => (
                403,
                json!({
                    "message": "access forbidden"
                }),
            ),
            GreaseError::NotFound => (
                404,
                json!({
                    "message": "resource not found"
                }),
            ),
            GreaseError::BadRequest(reason) => (
                400,
                json!({
                    "message": "bad request",
                    "reason": reason
                }),
            ),
            GreaseError::ServerError(error) => (
                500,
                json!({
                    "message": "server error",
                    "error": error
                }),
            ),
            GreaseError::DbError(error) => (
                500,
                json!({
                    "message": "database error",
                    "error": format!("{:?}", error)
                }),
            ),
            GreaseError::FromRowError(error) => (
                500,
                json!({
                    "message": "database error (error deserializing from returned row)",
                    "error": format!("{:?}", error)
                }),
            ),
        }
    }
}

#[cfg(test)]
impl std::cmp::PartialEq for GreaseError {
    fn eq(&self, other: &Self) -> bool {
        self.as_response() == other.as_response()
    }
}
