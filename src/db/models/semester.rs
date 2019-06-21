use chrono::{Duration, Local};
use db::models::*;
use db::traits::*;
use error::*;
use mysql::{prelude::GenericConnection, Conn};
use pinto::query_builder::*;

impl Semester {
    pub fn load(semester_name: &str, conn: &mut Conn) -> GreaseResult<Semester> {
        Self::first(
            &format!("name = '{}'", semester_name),
            conn,
            format!("No semester with name {}", semester_name),
        )
    }

    pub fn load_current<G: GenericConnection>(conn: &mut G) -> GreaseResult<Semester> {
        Self::first("current = true", conn, "No current semester set".to_owned())
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<Semester>> {
        Self::query_all_in_order(vec![("start_date", Order::Desc)], conn)
    }

    pub fn load_most_recent(conn: &mut Conn) -> GreaseResult<Semester> {
        match Self::load_all(conn)?.into_iter().next() {
            Some(semester) => Ok(semester),
            None => {
                let now = Local::now().naive_local();
                let new_semester = Semester {
                    name: "New Semester".to_owned(),
                    start_date: now,
                    end_date: now + Duration::weeks(12),
                    gig_requirement: 5,
                    current: true,
                };
                new_semester.insert(conn)?;
                Ok(new_semester)
            }
        }
    }

    pub fn create(new_semester: NewSemester, conn: &mut Conn) -> GreaseResult<String> {
        new_semester.insert(conn).map(|_| new_semester.name)
    }

    pub fn set_current(name: &str, conn: &mut Conn) -> GreaseResult<()> {
        let semester = Semester::load(name, conn)?;
        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;
        let query = Update::new(Self::table_name())
            .set("current", "false")
            .build();
        transaction.query(query).map_err(GreaseError::DbError)?;

        let query = Update::new(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .set("current", "true")
            .build();
        transaction.query(query).map_err(GreaseError::DbError)?;

        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn update(
        name: &str,
        updated_semester: &SemesterUpdate,
        conn: &mut Conn,
    ) -> GreaseResult<()> {
        let query = Update::new(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .set("name", &format!("'{}'", updated_semester.name))
            .set(
                "start_date",
                &updated_semester.start_date.to_value().as_sql(false),
            )
            .set(
                "end_date",
                &updated_semester.end_date.to_value().as_sql(false),
            )
            .set(
                "gig_requirement",
                &updated_semester.gig_requirement.to_string(),
            )
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(name: &str, conn: &mut Conn) -> GreaseResult<String> {
        let current_semester = Semester::load_current(conn)?;
        let query = Delete::new(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        if &current_semester.name == name {
            Semester::load_most_recent(conn).map(|semester| semester.name)
        } else {
            Ok(current_semester.name)
        }
    }
}
