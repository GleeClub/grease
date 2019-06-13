use crate::error::{GreaseError, GreaseResult};
use db::models::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use crate::util::random_base64;

impl GoogleDoc {
    pub fn load(doc_name: &str, conn: &MysqlConnection) -> GreaseResult<GoogleDoc> {
        use db::schema::google_docs::dsl::*;

        google_docs
            .filter(name.eq(doc_name))
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<GoogleDoc>> {
        use db::schema::google_docs::dsl::*;

        google_docs.load(conn).map_err(GreaseError::DbError)
    }

    pub fn insert(new_doc: &GoogleDoc, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::insert_into(google_docs)
            .values(new_doc)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn update(old_name: &str, changed_doc: &GoogleDoc, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        let rows_affected = diesel::update(google_docs.filter(name.eq(old_name)))
            .set((name.eq(&changed_doc.name), url.eq(&changed_doc.url)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        if rows_affected == 0 {
            Err(GreaseError::BadRequest(format!("no google doc named {}", old_name)))
        } else {
            Ok(())
        }
    }
}

impl Announcement {
    pub fn load(given_id: i32, conn: &MysqlConnection) -> GreaseResult<Announcement> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(id.eq(given_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!("no announcement with id {}", given_id)))
    }

    pub fn insert(new_content: &str, given_member: &str, given_semester: &str, conn: &MysqlConnection) -> GreaseResult<i32> {
        use db::schema::announcement::dsl::*;

        diesel::insert_into(announcement)
            .values((
                member.eq(given_member),
                content.eq(new_content),
                semester.eq(given_semester),
                archived.eq(false),
            ))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        announcement
            .order(id.desc())
            .select(id)
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .order(time)
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_semester(given_semester: &str, conn: &MysqlConnection) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(semester.eq(given_semester))
            .order(time)
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl MediaType {
    pub fn load(type_name: &str, conn: &MysqlConnection) -> GreaseResult<MediaType> {
        use db::schema::media_type::dsl::*;

        media_type
            .filter(name.eq(type_name))
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<MediaType>> {
        use db::schema::media_type::dsl::*;

        media_type
            .order(order)
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Variable {
    pub fn load(given_key: &str, conn: &MysqlConnection) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        match variable
            .filter(key.eq(given_key))
            .first::<Variable>(conn)
            .optional()
        {
            Ok(Some(var)) => Ok(Some(var.value)),
            Ok(None) => Ok(None),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn set(
        given_key: String,
        new_value: String,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        match Variable::load(&given_key, conn)? {
            Some(val) => {
                diesel::update(variable.filter(key.eq(&given_key)))
                    .set(value.eq(new_value))
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;
                Ok(Some(val))
            }
            None => {
                diesel::insert_into(variable)
                    .values(&Variable {
                        key: given_key,
                        value: new_value,
                    })
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;
                Ok(None)
            }
        }
    }
}

impl Session {
    pub fn load(given_email: &str, conn: &MysqlConnection) -> GreaseResult<Option<Session>> {
        use db::schema::session::dsl::*;

        session
            .filter(member.eq(given_email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn delete(given_email: &str, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::session::dsl::*;

        diesel::delete(session.filter(member.eq(given_email)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn generate(given_email: &str, conn: &MysqlConnection) -> GreaseResult<String> {
        #[derive(Insertable)]
        #[table_name = "session"]
        struct NewSession<'a> {
            member: &'a str,
            key: &'a str,
        }

        let new_key: String = random_base64(32)?;
        diesel::insert_into(session::table)
            .values(&NewSession {
                member: given_email,
                key: &new_key,
            })
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(new_key)
    }
}
