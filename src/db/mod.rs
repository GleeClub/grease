use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;
use mysql::prelude::FromRow;
use mysql::{prelude::GenericConnection, Conn};

pub mod models;
pub mod traits;

impl Extract for Conn {
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        dotenv::var("DATABASE_URL")
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))
            .and_then(|db_url| Conn::new(db_url).map_err(GreaseError::DbError))
    }
}

pub fn load<T: FromRow, G: GenericConnection>(query: &str, conn: &mut G) -> GreaseResult<Vec<T>> {
    conn.query(query)
        .map_err(GreaseError::DbError)
        .and_then(|result| {
            result
                .map(|row| {
                    row.map_err(GreaseError::DbError)
                        .and_then(|row| T::from_row_opt(row).map_err(GreaseError::FromRowError))
                })
                .collect::<GreaseResult<Vec<T>>>()
        })
}
