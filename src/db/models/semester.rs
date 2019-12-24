use chrono::{Duration, Local};
use db::{Semester, NewSemester};
use error::*;
use diesel::Connection;
use db::schema::semester::dsl::*;

impl Semester {
    pub fn load<C: Connection>(semester_name: &str, conn: &mut C) -> GreaseResult<Semester> {
        semester.filter(name.eq(semester_name))
            .first(conn)
            .map_err(GreaseError::DbError)
            // format!("No semester with name {}", semester_name),
    }

    pub fn load_current<C: Connection>(conn: &mut C) -> GreaseResult<Semester> {
        semester.filter(current.eq(true))
            .first(conn)
            .map_err(GreaseError::DbError)
            // "No current semester set".to_owned(),
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Semester>> {
        semester.order_by(start_date.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_most_recent<C: Connection>(conn: &mut C) -> GreaseResult<Semester> {
        if let Some(recent_semester) = Semester::load_all(conn)?.into_iter().next() {
            Ok(recent_semester)
        } else {
            let now = Local::now().naive_local();
            let new_semester = Semester {
                name: "New Semester".to_owned(),
                start_date: now,
                end_date: now + Duration::weeks(12),
                gig_requirement: 5,
                current: true,
            };

            diesel::insert_into(semester)
                .values(&new_semester)
                .execute(conn)
                .map(|_| new_semester)
                .map_err(GreaseError::DbError)
        }
    }

    pub fn create<C: Connection>(new_semester: NewSemester, conn: &mut C) -> GreaseResult<String> {
        Self::validate_new_semester(new_semester)?;

        diesel::insert_into(semester)
            .values(&new_semester)
            .execute(conn)
            .map(|_| new_semester.name)
            .map_err(GreaseError::DbError)
    }

    pub fn set_current<C: Connection>(given_name: &str, conn: &mut C) -> GreaseResult<()> {
        conn.transaction(|| {
            diesel::update(semester)
                .set(current.eq(false))
                .execute(conn)?;

                diesel::update(semester.filter(name.eq(given_name)))
                .set(current.eq(true))
                .execute(conn)
                // format!("No semester named '{}'.", name),
        })
        .map_err(GreaseError::DbError)
    }

    // TODO: add table for historical officership
    pub fn update<C: Connection>(
        given_name: &str,
        updated_semester: &NewSemester,
        conn: &mut C,
    ) -> GreaseResult<()> {
        Self::validate_new_semester(updated_semester)?;

        diesel::update(semester.filter(name.eq(given_name)))
            .set(updated_semester)
            .execute(conn)
            .map_err(GreaseError::DbError)
                // format!("No semester named '{}'.", &name),
    }

    pub fn delete<C: Connection>(given_name: &str, conn: &mut C) -> GreaseResult<String> {
        conn.transaction(|| {
            let all_semesters = Semester::load_all(conn)?;
            if all_semesters.len() == 1 && all_semesters.iter().any(|s| s.name == given_name) {
                return Err(GreaseError::BadRequest("You cannot delete the last semester.".to_owned()));
            } else if all_semesters.iter().all(|s| s.name != given_name) {
                return Err(GreaseError::BadRequest(format!("No semester exists with the name \"{}\".", given_name)));
            }

            diesel::delete(semester.filter(name.eq(given_name)))
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            if let Some(current_semester) = all_semesters.iter().find(|s| s.name != given_name && s.current) {
                Ok(current_semester.name.clone())
            } else {
                let first_semester = all_semesters.into_iter().find(|s| s.name != given_name).next().unwrap();
                diesel::update(semester.filter(name.eq(&first_semester.name)))
                    .set(current.eq(true))
                    .execute(conn)
                    .map(|_| first_semester.name)
                    .map_err(GreaseError::DbError)
            }
        })
    }

    fn validate_new_semester(new_semester: &NewSemester) -> GreaseResult<()> {
        if &new_semester.start_date >= &new_semester.end_date {
            Err(GreaseError::BadRequest(
                "The new semester must end after it begins.".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}
