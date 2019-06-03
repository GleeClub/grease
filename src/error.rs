use http::{response, header::{CONTENT_LENGTH, CONTENT_TYPE}};

pub enum GreaseError {
    NotFound,
    Unauthorized,
    Forbidden(Option<String>),
    ServerError(String),
    BadRequest(String),
    DbError(diesel::result::Error),
}

pub type GreaseResult<T> = Result<T, GreaseError>;

impl GreaseError {
    pub fn as_response(self) -> cgi::Response {
        let (status, message) = match self {
            GreaseError::Unauthorized => (401, "login required".to_owned()),
            GreaseError::Forbidden(Some(permission)) => (403, format!("access forbidden: user not allowed to {}", permission)),
            GreaseError::Forbidden(None) => (403, "access forbidden".to_owned()),
            GreaseError::NotFound => (404, "resource not found".to_owned()),
            GreaseError::BadRequest(reason) => (400, format!("bad request: {}", reason)),
            GreaseError::ServerError(error) => (405, format!("server error: {}", error)),
            GreaseError::DbError(error) => (401, format!("database error: {}", error)),
        };

        let body = message.into_bytes();
        response::Builder::new()
            .status(status)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, body.len().to_string().as_str())
            .body(body)
            .unwrap()
    }
}
