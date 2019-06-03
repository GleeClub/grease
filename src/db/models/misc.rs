use db::models::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use crate::error::{GreaseResult, GreaseError};

impl GoogleDoc {
    pub fn load(doc_name: &str, conn: &MysqlConnection) -> GreaseResult<GoogleDoc> {
        use db::schema::google_docs::dsl::*;

        google_docs.filter(name.eq(doc_name)).first(conn).map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<GoogleDoc> {
        use db::schema::google_docs::dsl::*;

        google_docs.load(conn).map_err(GreaseError::DbError)
    }
}

impl MediaType {
    pub fn load(type_name: &str, conn: &MysqlConnection) -> GreaseResult<MediaType> {
        use db::schema::media_type::dsl::*;

        media_type.filter(name.eq(type_name)).first(conn).map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<MediaType>> {
        use db::schema::media_type::dsl::*;

        media_type.order(order).load(conn).map_err(GreaseError::DbError)
    }
}

impl Variable {
    pub fn load(given_key: &str, conn: &MysqlConnection) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        match variable.filter(key.eq(given_key)).first(conn).optional() {
            Ok(Some(var)) => Ok(var.value),
            Ok(None) => Ok(None),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn set(given_key: &str, new_value: &str, conn: &MysqlConnection) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        let old_val = Variable::load(given_key, conn)?;

        diesel::insert_into(variable)
            .values(&Variable { key: given_key, value: new_value })
            .on_conflict(key)
            .do_update()
            .set(value.eq(new_value))
            .execute(&conn)
            .map_err(|err| GreaseError::DbError)?;
    
        Ok(old_val)
    }
}
