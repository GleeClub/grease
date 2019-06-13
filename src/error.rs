use db::models::Member;
use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    response,
};
use serde_json::json;

pub enum GreaseError {
    NotFound,
    AlreadyLoggedIn(String),
    Unauthorized,
    InvalidMethod,
    NotActiveYet(Member),
    Forbidden(Option<String>),
    ServerError(String),
    BadRequest(String),
    DbError(diesel::result::Error),
}

pub type GreaseResult<T> = Result<T, GreaseError>;

impl GreaseError {
    pub fn as_response(self) -> cgi::Response {
        let (status, response_body) = match self {
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
        };

        let body = response_body.to_string().into_bytes();
        response::Builder::new()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, body.len().to_string().as_str())
            .body(body)
            .unwrap()
    }
}
