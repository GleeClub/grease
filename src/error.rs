//! Error handling for the API.
//!
//! In development, feel free to add a variant to the GreaseError enum
//! to better format errors. This is always better than just forcing it
//! into a `BadRequest` or a generic `ServerError`. Make sure when doing
//! so to add adequate documentation.

use db::Member;
use serde_json::{json, Value};

/// The error enum for all error handling across the API.
///
/// See each variant for its corresponding error status code
/// and JSON error bodies.
#[derive(Debug, PartialEq)]
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
    DbError(diesel::result::Error),
    /// \[500\] An error occurred while connecting to the database.
    ///
    /// ```json
    /// {
    ///     "message": "error connecting to database",
    ///     "statusCode": 500,
    ///     "error": <error message>
    /// }
    /// ```
    ConnectionError(diesel::ConnectionError),
}

/// The return type for all endpoints.
pub type GreaseResult<T> = Result<T, GreaseError>;

impl GreaseError {
    pub fn status(&self) -> u16 {
        match self {
            GreaseError::Unauthorized | GreaseError::NotActiveYet(_) => 401,
            GreaseError::BadRequest(_) | GreaseError::AlreadyLoggedIn(_) => 400,
            GreaseError::Forbidden(_) => 403,
            GreaseError::NotFound => 404,
            GreaseError::ServerError(_)
            | GreaseError::DbError(_)
            | GreaseError::ConnectionError(_) => 500,
        }
    }

    pub fn as_response(&self) -> (u16, Value) {
        let mut json_val = match self {
            GreaseError::Unauthorized => json!({}),
            GreaseError::NotActiveYet(member) => json!({ "member": member }),
            GreaseError::AlreadyLoggedIn(token) => json!({ "token": token }),
            GreaseError::Forbidden(Some(permission)) => json!({ "requiredPermission": permission }),
            GreaseError::Forbidden(None) => json!({}),
            GreaseError::NotFound => json!({}),
            GreaseError::BadRequest(reason) => json!({ "reason": reason }),
            GreaseError::ServerError(error) => json!({ "error": error }),
            GreaseError::DbError(error) => json!({ "error": error.to_string() }),
            GreaseError::ConnectionError(error) => json!({ "error": error.to_string() }),
        };

        let status_code = self.status();
        json_val["statusCode"] = json!(status_code);
        json_val["message"] = json!(self.to_string());

        (status_code, json_val)
    }
}

impl std::convert::From<diesel::result::Error> for GreaseError {
    fn from(error: diesel::result::Error) -> GreaseError {
        GreaseError::DbError(error)
    }
}

impl std::fmt::Display for GreaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            GreaseError::Unauthorized => "login required",
            GreaseError::NotActiveYet(_) => "member not active yet",
            GreaseError::AlreadyLoggedIn(_) => "member already logged in",
            GreaseError::Forbidden(_) => "access forbidden",
            GreaseError::NotFound => "resource not found",
            GreaseError::BadRequest(_) => "bad request",
            GreaseError::ServerError(_) => "server error",
            GreaseError::DbError(_) => "database error",
            GreaseError::ConnectionError(_) => "error connecting to database",
        };

        write!(f, "{}", description)
    }
}
