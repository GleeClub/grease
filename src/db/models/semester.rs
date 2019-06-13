use chrono::{Duration, Local};
use db::models::{NewSemester, Semester};
use db::schema::semester::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::{GreaseError, GreaseResult};

impl Semester {
    pub fn load(semester_name: &str, conn: &MysqlConnection) -> GreaseResult<Semester> {
        semester
            .filter(name.eq(semester_name))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "No semester with name {}",
                semester_name
            )))
    }

    pub fn load_current(conn: &MysqlConnection) -> GreaseResult<Semester> {
        semester
            .filter(current.eq(true))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(
                "No current semester set".to_owned(),
            ))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Semester>> {
        semester
            .order_by(start_date.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_most_recent(conn: &MysqlConnection) -> GreaseResult<Semester> {
        if let Some(recent_semester) = semester
            .order_by(start_date.desc())
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
        {
            Ok(recent_semester)
        } else {
            let now = Local::now().naive_local();
            let new_semester = NewSemester {
                name: "New Semester".to_owned(),
                start_date: now,
                end_date: now + Duration::weeks(12),
            };
            let new_name = Semester::create(new_semester, conn)?;
            Semester::load(&new_name, conn)
        }
    }

    pub fn create(new_semester: NewSemester, conn: &MysqlConnection) -> GreaseResult<String> {
        diesel::insert_into(semester)
            .values((
                name.eq(&new_semester.name),
                start_date.eq(&new_semester.start_date),
                end_date.eq(&new_semester.end_date),
            ))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(new_semester.name)
    }
}
