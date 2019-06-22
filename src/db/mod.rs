use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;
use mysql::prelude::{FromRow, ToValue};
use mysql::{prelude::GenericConnection, Conn};

pub mod models;
pub mod traits;

impl Extract for Conn {
    #[cfg(not(test))]
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        dotenv::var("DATABASE_URL")
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))
            .and_then(|db_url| Conn::new(db_url).map_err(GreaseError::DbError))
    }

    #[cfg(test)]
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        dotenv::var("TEST_DATABASE_URL")
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))
            .and_then(|db_url| {
                let mut conn = Conn::new(db_url).map_err(GreaseError::DbError)?;
                conn.query("START TRANSACTION;").map_err(GreaseError::DbError)?;
                Ok(conn)
            })
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

pub fn to_value<'a, T: ToValue>(t: T) -> String {
    t.to_value().as_sql(false)
}
