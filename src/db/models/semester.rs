use chrono::{Duration, Local};
use db::*;
use error::*;
use pinto::query_builder::*;

impl Semester {
    pub fn load<C: Connection>(semester_name: &str, conn: &mut C) -> GreaseResult<Semester> {
        conn.first(
            &Semester::filter(&format!("name = '{}'", semester_name)),
            format!("No semester with name {}", semester_name),
        )
    }

    pub fn load_current<C: Connection>(conn: &mut C) -> GreaseResult<Semester> {
        conn.first(
            &Semester::filter("current = true"),
            "No current semester set".to_owned(),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Semester>> {
        conn.load(&Semester::select_all_in_order("start_date", Order::Desc))
    }

    pub fn load_most_recent<C: Connection>(conn: &mut C) -> GreaseResult<Semester> {
        if let Some(semester) =
            conn.first_opt(&Semester::select_all_in_order("start_date", Order::Desc))?
        {
            Ok(semester)
        } else {
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

    pub fn create<C: Connection>(new_semester: NewSemester, conn: &mut C) -> GreaseResult<String> {
        if &new_semester.start_date >= &new_semester.end_date {
            Err(GreaseError::BadRequest(
                "The new semester must end after it begins.".to_owned(),
            ))
        } else {
            new_semester.insert(conn)?;

            Ok(new_semester.name)
        }
    }

    pub fn set_current(name: &str, conn: &mut DbConn) -> GreaseResult<()> {
        conn.transaction(|transaction| {
            transaction.update(
                &Update::new(Semester::table_name()).set("current", "false"),
                "No semesters currently exist.".to_owned(),
            )?;

            transaction.update(
                &Update::new(Semester::table_name())
                    .filter(&format!("name = '{}'", name))
                    .set("current", "true"),
                format!("No semester named '{}'.", name),
            )
        })
    }

    // TODO: add table for historical officership
    pub fn update<C: Connection>(
        name: &str,
        updated_semester: &SemesterUpdate,
        conn: &mut C,
    ) -> GreaseResult<()> {
        if &updated_semester.start_date >= &updated_semester.end_date {
            Err(GreaseError::BadRequest(
                "The new semester must end after it begins.".to_owned(),
            ))
        } else {
            conn.update(
                &Update::new(Semester::table_name())
                    .filter(&format!("name = '{}'", name))
                    .set("start_date", &to_value(updated_semester.start_date))
                    .set("end_date", &to_value(updated_semester.end_date))
                    .set(
                        "gig_requirement",
                        &to_value(updated_semester.gig_requirement),
                    ),
                format!("No semester named '{}'.", name),
            )
        }
    }

    pub fn delete<C: Connection>(name: &str, conn: &mut C) -> GreaseResult<String> {
        conn.delete(
            &Delete::new(Semester::table_name()).filter(&format!("name = '{}'", name)),
            format!("No semester named '{}'.", name),
        )?;

        if let Some(current_semester) =
            conn.first_opt::<Semester>(&Semester::filter("current = true"))?
        {
            Ok(current_semester.name)
        } else {
            Semester::load_most_recent(conn).map(|semester| semester.name)
        }
    }
}
