use crate::error::GreaseResult;
use mysql::prelude::FromRow;
use pinto::query_builder::*;

#[cfg(not(test))]
pub use self::conn::*;
#[cfg(test)]
pub use self::test_conn::*;

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

#[cfg(not(test))]
mod conn {
    use super::Connection;
    use crate::error::{GreaseError, GreaseResult};
    use crate::extract::Extract;
    use mysql::prelude::FromRow;
    use mysql::{prelude::GenericConnection, Conn};
    use pinto::query_builder::*;

    pub struct DbConn {
        conn: Conn,
    }

    impl Extract for DbConn {
        fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
            DbConn::from_env_var("DATABASE_URL")
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

        pub fn transaction<T, F: FnOnce(&mut DbTransaction) -> GreaseResult<T>>(
            &mut self,
            callback: F,
        ) -> GreaseResult<T> {
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
            let transaction = conn
                .start_transaction(false, None, None)
                .map_err(GreaseError::DbError)?;
            Ok(DbTransaction { conn: transaction })
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

    fn first_with_conn<T: FromRow, G: GenericConnection>(
        query: &Select,
        error_message: String,
        conn: &mut G,
    ) -> GreaseResult<T> {
        conn.first(query.build())
            .map_err(GreaseError::DbError)
            .and_then(|first| first.ok_or(GreaseError::BadRequest(error_message)))
    }

    fn first_opt_with_conn<T: FromRow, G: GenericConnection>(
        query: &Select,
        conn: &mut G,
    ) -> GreaseResult<Option<T>> {
        conn.first(query.build()).map_err(GreaseError::DbError)
    }

    fn load_with_conn<T: FromRow, G: GenericConnection>(
        query: &Select,
        conn: &mut G,
    ) -> GreaseResult<Vec<T>> {
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

    fn load_as_with_conn<T: FromRow + Into<I>, I, G: GenericConnection>(
        query: &Select,
        conn: &mut G,
    ) -> GreaseResult<Vec<I>> {
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

    pub fn insert_multiple_with_conn<G: GenericConnection>(
        queries: Vec<&Insert>,
        conn: &mut G,
    ) -> GreaseResult<()> {
        for query in queries {
            conn.query(query.build()).map_err(GreaseError::DbError)?;
        }
        Ok(())
    }

    pub fn insert_returning_id_with_conn<G: GenericConnection>(
        query: &Insert,
        conn: &mut G,
    ) -> GreaseResult<i32> {
        conn.query(query.build())
            .map_err(GreaseError::DbError)
            .map(|result| result.last_insert_id() as i32)
    }

    fn update_with_conn<G: GenericConnection>(
        query: &Update,
        conn: &mut G,
        error_message: String,
    ) -> GreaseResult<()> {
        if conn
            .query(query.build())
            .map_err(GreaseError::DbError)?
            .affected_rows()
            > 0
        {
            Ok(())
        } else {
            Err(GreaseError::ServerError(error_message))
        }
    }

    fn update_opt_with_conn<G: GenericConnection>(query: &Update, conn: &mut G) -> GreaseResult<()> {
        conn.query(query.build()).map_err(GreaseError::DbError)?;
        Ok(())
    }

    fn delete_with_conn<G: GenericConnection>(
        query: &Delete,
        conn: &mut G,
        error_message: String,
    ) -> GreaseResult<()> {
        if conn
            .query(query.build())
            .map_err(GreaseError::DbError)?
            .affected_rows()
            > 0
        {
            Ok(())
        } else {
            Err(GreaseError::ServerError(error_message))
        }
    }

    fn delete_opt_with_conn<G: GenericConnection>(query: &Delete, conn: &mut G) -> GreaseResult<()> {
        conn.query(query.build()).map_err(GreaseError::DbError)?;
        Ok(())
    }
}

#[cfg(test)]
mod test_conn {
    extern crate mysql_common;
    extern crate smallvec;

    use super::Connection;
    use crate::error::{GreaseError, GreaseResult};
    use crate::extract::Extract;
    use mocktopus::macros::*;
    use mysql::prelude::FromRow;
    use pinto::query_builder::*;

    pub struct DbConn {
        queries: Vec<(String, serde_json::Value)>,
    }

    impl Extract for DbConn {
        fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
            panic!("DbConn must always be mocked when testing extract");
        }
    }

    #[mockable]
    impl DbConn {
        pub fn setup(queries: Vec<(&str, serde_json::Value)>) -> DbConn {
            DbConn {
                queries: queries.into_iter()
                    .map(|(query, data)| (query.to_string(), data))
                    .collect(),
            }
        }

        pub fn transaction<T, F: FnOnce(&mut DbTransaction) -> GreaseResult<T>>(
            &mut self,
            callback: F,
        ) -> GreaseResult<T> {
            let mut transaction = DbTransaction::from_conn(self);
            callback(&mut transaction)
        }

        pub fn check_query(&mut self, expected_query: &str) -> Vec<mysql::Row> {
            if self.queries.len() > 0 {
                let (query, to_return) = self.queries.remove(0);
                assert_eq!(&query, expected_query);
                Self::json_to_rows(&to_return)
            } else {
                panic!("too many queries were made");
            }
        }

        pub fn json_to_rows(value: &serde_json::Value) -> Vec<self::mysql_common::row::Row> {
            match value {
                serde_json::Value::Array(rows) => {
                    rows.iter().map(|row| Self::json_to_row(row)).collect()
                }
                serde_json::Value::Object(_map) => vec![Self::json_to_row(value)],
                serde_json::Value::Null => vec![],
                _other => panic!("JSON can only be a single row object, multiple row objects, or null"),
            }
        }

        fn json_to_row(value: &serde_json::Value) -> self::mysql_common::row::Row {
            use self::smallvec::SmallVec;

            let (columns, values) = value
                .as_object()
                .expect("JSON row wasn't an object")
                .iter()
                .map(|(key, value)| {
                    let column = Self::column_from_str(key);
                    let value = Self::json_to_mysql(value);
                    (column, value)
                })
                .fold(
                    (Vec::new(), SmallVec::new()),
                    |(mut columns, mut values), (column, value)| {
                        columns.push(column);
                        values.push(value);
                        (columns, values)
                    },
                );

            self::mysql_common::row::new_row(values, std::sync::Arc::new(columns))
        }

        fn json_to_mysql(value: &serde_json::Value) -> mysql::Value {
            use regex::Regex;

            match value {
                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                    panic!("row elements not allowed to be objects or arrays")
                }
                serde_json::Value::Null => mysql::Value::NULL,
                serde_json::Value::Number(number) => {
                    if number.is_f64() {
                        mysql::Value::Float(number.as_f64().unwrap())
                    } else if number.is_i64() {
                        mysql::Value::Int(number.as_i64().unwrap())
                    } else {
                        mysql::Value::UInt(number.as_u64().unwrap())
                    }
                }
                serde_json::Value::Bool(boolean) => mysql::Value::Int(if *boolean { 1 } else { 0 }),
                serde_json::Value::String(string) => {
                    if let Some(matches) = Regex::new(r"^'(\d{4})-(\d{2})-(\d{2})'$")
                        .unwrap()
                        .captures(&string)
                    {
                        mysql::Value::Date(
                            matches[1].parse().unwrap(),
                            matches[2].parse().unwrap(),
                            matches[3].parse().unwrap(),
                            0,
                            0,
                            0,
                            0,
                        )
                    } else if let Some(matches) =
                        Regex::new(r"^'(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})'$")
                            .unwrap()
                            .captures(&string)
                    {
                        mysql::Value::Date(
                            matches[1].parse().unwrap(),
                            matches[2].parse().unwrap(),
                            matches[3].parse().unwrap(),
                            matches[4].parse().unwrap(),
                            matches[5].parse().unwrap(),
                            matches[6].parse().unwrap(),
                            0,
                        )
                    } else if let Some(matches) =
                        Regex::new(r"^'(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})\.(\d{6})'$")
                            .unwrap()
                            .captures(&string)
                    {
                        mysql::Value::Date(
                            matches[1].parse().unwrap(),
                            matches[2].parse().unwrap(),
                            matches[3].parse().unwrap(),
                            matches[4].parse().unwrap(),
                            matches[5].parse().unwrap(),
                            matches[6].parse().unwrap(),
                            matches[7].parse().unwrap(),
                        )
                    } else if let Some(matches) = Regex::new(r"^'(-?)(\d{3}):(\d{2}):(\d{2})'$")
                        .unwrap()
                        .captures(&string)
                    {
                        mysql::Value::Time(
                            &matches[1] == "-",
                            matches[2].parse::<u32>().unwrap() / 24,
                            (matches[2].parse::<u32>().unwrap() % 24) as u8,
                            matches[3].parse().unwrap(),
                            matches[4].parse().unwrap(),
                            0,
                        )
                    } else if let Some(matches) = Regex::new(r"^'(-?)(\d{3}):(\d{2}):(\d{2})\.(\d{6})'$")
                        .unwrap()
                        .captures(&string)
                    {
                        mysql::Value::Time(
                            &matches[1] == "-",
                            matches[2].parse::<u32>().unwrap() / 24,
                            (matches[2].parse::<u32>().unwrap() % 24) as u8,
                            matches[3].parse().unwrap(),
                            matches[4].parse().unwrap(),
                            matches[5].parse().unwrap(),
                        )
                    } else {
                        mysql::Value::Bytes(string.clone().into_bytes())
                    }
                }
            }
        }

        fn column_from_str(name: &str) -> self::mysql_common::packets::Column {
            // const COLUMN_PACKET: &[u8] =
            //     b"\x03def\x06schema\x05table\x09org_table\x04name\
            //       \x08org_name\x0c\x21\x00\x0F\x00\x00\x00\x00\x01\x00\x08\x00\x00";

            let column_packet_start = Box::new(b"\x03def\x06schema\x05table\x09org_table").to_vec();
            let column_packet_end =
                Box::new(b"\x08org_name\x0c\x21\x00\x0F\x00\x00\x00\x00\x01\x00\x08\x00\x00").to_vec();

            let packet = column_packet_start
                .into_iter()
                .chain(Some(name.len() as u8).into_iter())
                .chain(name.chars().map(|c| c as u8))
                .chain(column_packet_end.into_iter())
                .collect::<Vec<u8>>();

            self::mysql_common::packets::column_from_payload(packet)
                .expect("Error building column from packet")
        }

        fn from_row<T: FromRow>(row: mysql::Row) -> T {
            T::from_row_opt(row).expect("incorrectly formed row supplied")
        }

        pub fn compare_result<T: PartialEq + std::fmt::Debug>(self, left: T, right: T) {
            if self.queries.len() > 0 {
                panic!("Not all queries were made");
            }

            assert_eq!(left, right);
        }
    }

    impl Connection for DbConn {
        fn query(&mut self, query: String) -> GreaseResult<()> {
            self.check_query(&query);
            Ok(())
        }

        fn first<T: FromRow>(&mut self, query: &Select, error_message: String) -> GreaseResult<T> {
            if let Some(row) = self.check_query(&query.build()).into_iter().nth(0) {
                Ok(Self::from_row(row))
            } else {
                Err(GreaseError::ServerError(error_message))
            }
        }

        fn first_opt<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Option<T>> {
            Ok(self
                .check_query(&query.build())
                .into_iter()
                .next()
                .map(|row| Self::from_row(row)))
        }

        fn load<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Vec<T>> {
            let rows = self.check_query(&query.build());
            Ok(rows.into_iter().map(|row| Self::from_row(row)).collect())
        }

        fn load_as<T: FromRow + Into<I>, I>(&mut self, query: &Select) -> GreaseResult<Vec<I>> {
            let rows = self.check_query(&query.build());
            Ok(rows
                .into_iter()
                .map(|row| Self::from_row::<T>(row).into())
                .collect())
        }

        fn insert(&mut self, query: &Insert) -> GreaseResult<()> {
            self.check_query(&query.build());
            Ok(())
        }

        fn insert_multiple(&mut self, queries: Vec<&Insert>) -> GreaseResult<()> {
            for query in queries {
                self.check_query(&query.build());
            }
            Ok(())
        }

        fn insert_returning_id(&mut self, query: &Insert) -> GreaseResult<i32> {
            let row = self
                .check_query(&query.build())
                .into_iter()
                .next()
                .expect("test should have returned an id");
            Ok(Self::from_row(row))
        }

        fn update(&mut self, query: &Update, _error_message: String) -> GreaseResult<()> {
            self.check_query(&query.build());
            Ok(())
        }

        fn update_opt(&mut self, query: &Update) -> GreaseResult<()> {
            self.check_query(&query.build());
            Ok(())
        }

        fn delete(&mut self, query: &Delete, _error_message: String) -> GreaseResult<()> {
            self.check_query(&query.build());
            Ok(())
        }

        fn delete_opt(&mut self, query: &Delete) -> GreaseResult<()> {
            self.check_query(&query.build());
            Ok(())
        }
    }

    pub struct DbTransaction<'a> {
        conn: &'a mut DbConn,
    }

    impl<'a> DbTransaction<'a> {
        pub fn from_conn(conn: &'a mut DbConn) -> Self {
            DbTransaction { conn }
        }
    }

    impl<'a> Connection for DbTransaction<'a> {
        fn query(&mut self, query: String) -> GreaseResult<()> {
            self.conn.check_query(&query);
            Ok(())
        }

        fn first<T: FromRow>(&mut self, query: &Select, error_message: String) -> GreaseResult<T> {
            if let Some(row) = self.conn.check_query(&query.build()).into_iter().nth(0) {
                Ok(DbConn::from_row(row))
            } else {
                Err(GreaseError::ServerError(error_message))
            }
        }

        fn first_opt<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Option<T>> {
            Ok(self.conn
                .check_query(&query.build())
                .into_iter()
                .next()
                .map(|row| DbConn::from_row(row)))
        }

        fn load<T: FromRow>(&mut self, query: &Select) -> GreaseResult<Vec<T>> {
            let rows = self.conn.check_query(&query.build());
            Ok(rows.into_iter().map(|row| DbConn::from_row(row)).collect())
        }

        fn load_as<T: FromRow + Into<I>, I>(&mut self, query: &Select) -> GreaseResult<Vec<I>> {
            let rows = self.conn.check_query(&query.build());
            Ok(rows
                .into_iter()
                .map(|row| DbConn::from_row::<T>(row).into())
                .collect())
        }

        fn insert(&mut self, query: &Insert) -> GreaseResult<()> {
            self.conn.check_query(&query.build());
            Ok(())
        }

        fn insert_multiple(&mut self, queries: Vec<&Insert>) -> GreaseResult<()> {
            for query in queries {
                self.conn.check_query(&query.build());
            }
            Ok(())
        }

        fn insert_returning_id(&mut self, query: &Insert) -> GreaseResult<i32> {
            let row = self.conn
                .check_query(&query.build())
                .into_iter()
                .next()
                .expect("test should have returned an id");
            Ok(DbConn::from_row(row))
        }

        fn update(&mut self, query: &Update, _error_message: String) -> GreaseResult<()> {
            self.conn.check_query(&query.build());
            Ok(())
        }

        fn update_opt(&mut self, query: &Update) -> GreaseResult<()> {
            self.conn.check_query(&query.build());
            Ok(())
        }

        fn delete(&mut self, query: &Delete, _error_message: String) -> GreaseResult<()> {
            self.conn.check_query(&query.build());
            Ok(())
        }

        fn delete_opt(&mut self, query: &Delete) -> GreaseResult<()> {
            self.conn.check_query(&query.build());
            Ok(())
        }
    }

}
