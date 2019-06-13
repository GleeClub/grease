use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;

pub mod models;
pub mod schema;

impl Extract for MysqlConnection {
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        return Err(GreaseError::ServerError("testing this out...".to_owned()));
        let url = dotenv::var("DATABASE_URL")
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))?
            .to_string();
        MysqlConnection::establish(&url).map_err(|err| {
            GreaseError::ServerError(format!("couldn't connect to database: {}", err))
        })
    }
}
