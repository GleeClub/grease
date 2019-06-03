use crate::error::{GreaseResult, GreaseError};
use serde::Deserialize;
use diesel::mysql::MysqlConnection;

pub trait Extract: Sized {
    fn extract(request: &cgi::Request) -> GreaseResult<Self>;
}

// impl<T: Extract, U: Extract> Extract for (T, U) {
//     fn extract(request: &cgi::Request) -> GreaseResult<Self> {
//         Ok((T::extract(request)?, U::extract(request)?))
//     }
// }

// impl<T: Extract, U: Extract, V: Extract> Extract for (T, U, V) {
//     fn extract(request: &cgi::Request) -> GreaseResult<Self> {
//         Ok((T::extract(request)?, U::extract(request)?, V::extract(request)?))
//     }
// }

// impl<T: Extract, U: Extract, V: Extract, W: Extract> Extract for (T, U, V, W) {
//     fn extract(request: &cgi::Request) -> GreaseResult<Self> {
//         Ok((T::extract(request)?, U::extract(request)?, V::extract(request)?, W::extract(request)?))
//     }
// }

impl Extract for MysqlConnection {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        let url = dotenv::var("DATABASE_URL")
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))?
            .to_string();
        MysqlConnection::establish(&url).map_err(|err| GreaseError::ServerError(format!("couldn't connect to database: {}", err)))
    }
}
