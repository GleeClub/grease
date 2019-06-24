use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;
use mysql::prelude::FromRow;
use mysql::{prelude::GenericConnection, Conn};
use pinto::query_builder::*;

pub trait Connection {
    fn query(&mut self, query: String) -> GreaseResult<()>;

    fn first<T: FromRow>(&mut self, query: &Select, error_message: String) -> GreaseResult<T>;

    fn first_opt<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Option<T>>;

    fn load<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Vec<T>>;

    fn load_as<T: FromRow + Into<I>, I>(&mut self, query: &Select) -> GreaseResult<Vec<I>>;

    fn insert(&mut self, query: &Insert) -> GreaseResult<()>;

    fn insert_multiple(&mut self, queries: Vec<&Insert>) -> GreaseResult<()>;

    fn insert_returning_id(&mut self, query: &Insert) -> GreaseResult<i32>;

    fn update(&mut self, query: &Update, error_message: String) -> GreaseResult<()>;

    fn update_opt(&mut self, query: &Update) -> GreaseResult<()>;

    fn delete(&mut self, query: &Delete, error_message: String) -> GreaseResult<()>;

    fn delete_opt(&mut self, query: &Delete) -> GreaseResult<()>;
}

pub struct DbConn {
    conn: Conn,
}

impl Extract for DbConn {
    #[cfg(not(test))]
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        DbConn::from_env_var("DATABASE_URL")
    }

    #[cfg(test)]
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        unimplemented!()
    }
}

impl DbConn {
    pub fn from_env_var(db_url_var_name: &'static str) -> GreaseResult<DbConn> {
        dotenv::dotenv().ok();
        let db_url = std::env::var(db_url_var_name)
            .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))?;
        let conn = Conn::new(db_url).map_err(GreaseError::DbError)?;

        Ok(DbConn { conn })
    }

    pub fn transaction<T, F: FnOnce(&mut DbTransaction) -> GreaseResult<T>>(&mut self, callback: F) -> GreaseResult<T> {
        let mut transaction = DbTransaction::from_conn(&mut self.conn)?;
        let returned = callback(&mut transaction)?;
        transaction.conn.commit().map_err(GreaseError::DbError)?;

        Ok(returned)
    }
}

impl Connection for DbConn {
    fn query(&mut self, query: String) -> GreaseResult<()> {
        self.conn.query(query).map_err(GreaseError::DbError)?;
        Ok(())
    }

    fn first<T: FromRow>(&mut self, query: &Select, error_message: String) -> GreaseResult<T> {
        first_with_conn(query, error_message, &mut self.conn)
    }

    fn first_opt<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Option<T>> {
        first_opt_with_conn(query, &mut self.conn)
    }

    fn load<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Vec<T>> {
        load_with_conn(query, &mut self.conn)
    }

    fn load_as<T: FromRow + Into<I>, I>(&mut self, query: &Select) -> GreaseResult<Vec<I>> {
        load_as_with_conn::<T, I, _>(query, &mut self.conn)
    }

    fn insert(&mut self, query: &Insert) -> GreaseResult<()> {
        insert_with_conn(query, &mut self.conn)
    }

    fn insert_multiple(&mut self, queries: Vec<&Insert>) -> GreaseResult<()> {
        insert_multiple_with_conn(queries, &mut self.conn)
    }

    fn insert_returning_id(&mut self, query: &Insert) -> GreaseResult<i32> {
        insert_returning_id_with_conn(query, &mut self.conn)
    }

    fn update(&mut self, query: &Update, error_message: String) -> GreaseResult<()> {
        update_with_conn(query, &mut self.conn, error_message)
    }

    fn update_opt(&mut self, query: &Update) -> GreaseResult<()> {
        update_opt_with_conn(query, &mut self.conn)
    }

    fn delete(&mut self, query: &Delete, error_message: String) -> GreaseResult<()> {
        delete_with_conn(query, &mut self.conn, error_message)
    }

    fn delete_opt(&mut self, query: &Delete) -> GreaseResult<()> {
        delete_opt_with_conn(query, &mut self.conn)
    }
}

pub struct DbTransaction<'a> {
    conn: mysql::Transaction<'a>,
}

impl<'a> DbTransaction<'a> {
    pub fn from_conn(conn: &'a mut Conn) -> GreaseResult<Self> {
        let transaction = conn.start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;
        Ok(DbTransaction {
            conn: transaction,
        })
    }
}

impl<'a> Connection for DbTransaction<'a> {
    fn query(&mut self, query: String) -> GreaseResult<()> {
        self.conn.query(query).map_err(GreaseError::DbError)?;
        Ok(())
    }

    fn first<T: FromRow>(&mut self, query: &Select, error_message: String) -> GreaseResult<T> {
        first_with_conn(query, error_message, &mut self.conn)
    }

    fn first_opt<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Option<T>> {
        first_opt_with_conn(query, &mut self.conn)
    }

    fn load<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Vec<T>> {
        load_with_conn(query, &mut self.conn)
    }

    fn load_as<T: FromRow + Into<I>, I>(&mut self, query: &Select) -> GreaseResult<Vec<I>> {
        load_as_with_conn::<T, I, _>(query, &mut self.conn)
    }

    fn insert(&mut self, query: &Insert) -> GreaseResult<()> {
        insert_with_conn(query, &mut self.conn)
    }

    fn insert_multiple(&mut self, queries: Vec<&Insert>) -> GreaseResult<()> {
        insert_multiple_with_conn(queries, &mut self.conn)
    }

    fn insert_returning_id(&mut self, query: &Insert) -> GreaseResult<i32> {
        insert_returning_id_with_conn(query, &mut self.conn)
    }

    fn update(&mut self, query: &Update, error_message: String) -> GreaseResult<()> {
        update_with_conn(query, &mut self.conn, error_message)
    }

    fn update_opt(&mut self, query: &Update) -> GreaseResult<()> {
        update_opt_with_conn(query, &mut self.conn)
    }

    fn delete(&mut self, query: &Delete, error_message: String) -> GreaseResult<()> {
        delete_with_conn(query, &mut self.conn, error_message)
    }

    fn delete_opt(&mut self, query: &Delete) -> GreaseResult<()> {
        delete_opt_with_conn(query, &mut self.conn)
    }
}

fn first_with_conn<T: FromRow, G: GenericConnection>(query: &Select, error_message: String, conn: &mut G) -> GreaseResult<T> {
    conn.first(query.build()).map_err(GreaseError::DbError).and_then(|first| first.ok_or(GreaseError::BadRequest(error_message)))
}

fn first_opt_with_conn<T: FromRow, G: GenericConnection>(query: &Select, conn: &mut G) -> GreaseResult<Option<T>> {
    conn.first(query.build()).map_err(GreaseError::DbError)
}

fn load_with_conn<T: FromRow, G: GenericConnection>(query: &Select, conn: &mut G) -> GreaseResult<Vec<T>> {
    conn.query(query.build())
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

fn load_as_with_conn<T: FromRow + Into<I>, I, G: GenericConnection>(query: &Select, conn: &mut G) -> GreaseResult<Vec<I>> {
    conn.query(query.build())
        .map_err(GreaseError::DbError)
        .and_then(|result| {
            result
                .map(|row| {
                    row.map_err(GreaseError::DbError)
                        .and_then(|row| T::from_row_opt(row).map_err(GreaseError::FromRowError))
                        .map(|val| val.into())
                })
                .collect::<GreaseResult<Vec<I>>>()
        })
}

pub fn insert_with_conn<G: GenericConnection>(query: &Insert, conn: &mut G) -> GreaseResult<()> {
    conn.query(query.build()).map_err(GreaseError::DbError)?;
    Ok(())
}

pub fn insert_multiple_with_conn<G: GenericConnection>(queries: Vec<&Insert>, conn: &mut G) -> GreaseResult<()> {
    for query in queries {
        conn.query(query.build()).map_err(GreaseError::DbError)?;
    }
    Ok(())
}

pub fn insert_returning_id_with_conn<G: GenericConnection>(query: &Insert, conn: &mut G) -> GreaseResult<i32> {
    conn.query(query.build()).map_err(GreaseError::DbError).map(|result| result.last_insert_id() as i32)
}

fn update_with_conn<G: GenericConnection>(query: &Update, conn: &mut G, error_message: String) -> GreaseResult<()> {
    if conn.query(query.build()).map_err(GreaseError::DbError)?.affected_rows() > 0 {
        Ok(())
    } else {
        Err(GreaseError::ServerError(error_message))
    }
}

fn update_opt_with_conn<G: GenericConnection>(query: &Update, conn: &mut G) -> GreaseResult<()> {
    conn.query(query.build()).map_err(GreaseError::DbError)?;
    Ok(())
}

fn delete_with_conn<G: GenericConnection>(query: &Delete, conn: &mut G, error_message: String) -> GreaseResult<()> {
    if conn.query(query.build()).map_err(GreaseError::DbError)?.affected_rows() > 0 {
        Ok(())
    } else {
        Err(GreaseError::ServerError(error_message))
    }
}

fn delete_opt_with_conn<G: GenericConnection>(query: &Delete, conn: &mut G) -> GreaseResult<()> {
    conn.query(query.build()).map_err(GreaseError::DbError)?;
    Ok(())
}
