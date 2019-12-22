//! Error handling for the API.
//!
//! In development, feel free to add a variant to the GreaseError enum
//! to better format errors. This is always better than just forcing it
//! into a `BadRequest` or a generic `ServerError`. Make sure when doing
//! so to add adequate documentation.

use db::models::Member;
use serde_json::{json, Value};

/// The error enum for all error handling across the API.
///
/// See each variant for its corresponding error status code
/// and JSON error bodies.
#[derive(Debug)]
pub enum GreaseError {
    /// \[404\] The endpoint was not found.
    ///
    /// ```json
    /// {
    ///     "message": "resource not found",
    ///     "statusCode": 404
    /// }
    /// ```
    NotFound,
    /// \[400\] The member has already been issued an API token.
    ///
    /// ```json
    /// {
    ///     "message": "member already logged in",
    ///     "statusCode": 400,
    ///     "token": <API token>
    /// }
    /// ```
    AlreadyLoggedIn(String),
    /// \[401\] The endpoint requires a logged-in member.
    ///
    /// ```json
    /// {
    ///     "message": "login required",
    ///     "statusCode": 401
    /// }
    /// ```
    Unauthorized,
    /// \[401\] The member in question is not active for the given semester.
    ///
    /// ```json
    /// {
    ///     "message": "member not active yet",
    ///     "statusCode": 401,
    ///     "member": Member
    /// }
    /// ```
    ///
    /// See [Member](crate::db::models::Member) for its JSON format.
    NotActiveYet(Member),
    /// \[403\] The current member does not have currently have permission to use the endpoint.
    ///
    /// ```json
    /// {
    ///     "message": "access forbidden",
    ///     "statusCode": 403,
    ///     "requiredPermission": <permission name>?
    /// }
    /// ```
    ///
    /// If the restricted action is available with a permission, the
    /// `requiredPermission` field will have the name of the permission. If
    /// not, the field will not exist.
    Forbidden(Option<String>),
    /// \[500\] An error occurred while handling the request.
    ///
    /// ```json
    /// {
    ///     "message": "server error",
    ///     "statusCode": 500,
    ///     "error": <error message>
    /// }
    /// ```
    ServerError(String),
    /// \[400\] The request to the API was malformed.
    ///
    /// ```json
    /// {
    ///     "message": "bad request",
    ///     "statusCode": 400,
    ///     "reason": <reason>
    /// }
    /// ```
    BadRequest(String),
    /// \[500\] An error occured while interacting with the database.
    ///
    /// ```json
    /// {
    ///     "message": "database error",
    ///     "statusCode": 500,
    ///     "error": <error message>
    /// }
    /// ```
    DbError(mysql::error::Error),
    /// \[500\] An error occurred while deserializing a MySQL row.
    ///
    /// ```json
    /// {
    ///     "message": "database error (error deserializing from returned row)",
    ///     "statusCode": 500,
    ///     "error": <error message>
    /// }
    /// ```
    FromRowError(mysql::FromRowError),
}

/// The return type for all endpoints.
pub type GreaseResult<T> = Result<T, GreaseError>;

impl GreaseError {
    /// Renders all error variants as JSON errors.
    ///
    /// See the enum variants for their respective formats.
    pub fn as_response(&self) -> (u16, Value) {
        match self {
            GreaseError::Unauthorized => (
                401,
                json!({
                    "message": "login required",
                    "statusCode": 401
                }),
            ),
            GreaseError::NotActiveYet(member) => (
                401,
                json!({
                    "message": "member not active yet",
                    "statusCode": 401,
                    "member": member.to_json()
                }),
            ),
            GreaseError::AlreadyLoggedIn(token) => (
                400,
                json!({
                    "message": "member already logged in",
                    "statusCode": 400,
                    "token": token
                }),
            ),
            GreaseError::Forbidden(Some(permission)) => (
                403,
                json!({
                    "message": "access forbidden",
                    "statusCode": 403,
                    "requiredPermission": permission
                }),
            ),
            GreaseError::Forbidden(None) => (
                403,
                json!({
                    "message": "access forbidden",
                    "statusCode": 403
                }),
            ),
            GreaseError::NotFound => (
                404,
                json!({
                    "message": "resource not found",
                    "statusCode": 404
                }),
            ),
            GreaseError::BadRequest(reason) => (
                400,
                json!({
                    "message": "bad request",
                    "statusCode": 400,
                    "reason": reason
                }),
            ),
            GreaseError::ServerError(error) => (
                500,
                json!({
                    "message": "server error",
                    "statusCode": 500,
                    "error": error
                }),
            ),
            GreaseError::DbError(error) => (
                500,
                json!({
                    "message": "database error",
                    "statusCode": 500,
                    "error": format!("{:?}", error)
                }),
            ),
            GreaseError::FromRowError(error) => (
                500,
                json!({
                    "message": "database error (error deserializing from returned row)",
                    "statusCode": 500,
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
