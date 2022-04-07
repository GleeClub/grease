use async_graphql::{ComplexObject, SimpleObject, Context, Enum, Result};
use crate::models::member::member::Member;
use crate::db_conn::DbConn;

#[derive(SimpleObject)]
pub struct ActiveSemester {
    /// The email of the member
    pub member: String,
    /// The name of the semester
    pub semester: String,
    /// Whether the member was registered for the class
    pub enrollment: Enrollment,
    /// What section the member sang in
    pub section: Option<String>,
}

#[ComplexObject]
impl ActiveSemester {
    /// The grades for the member in the given semester
    pub async fn grades(&self, ctx: &Context<'_>) -> Result<Grades> {
        // TODO
        // Grades.for_member (Member.with_email! @member), (Semester.with_name! @semester)
    }
}

#[derive(Enum)]
pub enum Enrollment {
    Class,
    Club,
}

impl ActiveSemester {
    pub async fn all_for_member(member: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM active_semester WHERE member = ?", member).query_all(conn).await
    }

    pub async fn for_member_during_semester(member: &str, semester: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM active_semester WHERE member = ? AND semester = ?", member, semester).query_optional(conn).await
    }

    pub async fn create_for_member(member: &Member, new_semester: NewActiveSemester, conn: &DbConn) -> Result<()> {
        if Member::semester(new_semester.semester, conn).await?.is_some() {
            return Err(format!("{} is already active for the current semester", member.full_name()));
        }

        // TODO: create attendance or something?

        sqlx::query!(
            "INSERT INTO active_semester (member, semester, enrollment, section) VALUES (?, ?, ?, ?)",
            member.email, new_semester.semester, new_semester.enrollment, new_semester.section
        ).query(conn).await
    }

    pub async fn update(semester_update: ActiveSemesterUpdate, conn: &DbConn) -> Result<()> {
        let active_semester = Self::for_semester(semester_update.member, semester_update.semester, conn).await?;

        match (semester_update.enrollment, active_semester) {
            (Some(enrollment), Some(active_semester)) => {
                sqlx::query!(
                    "UPDATE active_semester SET enrollment = ?, section = ? WHERE member = ? AND semester = ?",
                    enrollment, semester_update.section, semester_update.member, semester_update.semester
                ).query(conn).await
            }
            (Some(enrollment), None) => {
                sqlx::query!(
                    "INSERT INTO active_semester (member, semester, enrollment, section) VALUES (?, ?, ?, ?)",
                    semester_update.member, semester_update.semester, enrollment, semester_update.section
                ).query(conn).await
            }
            (None, Some(active_semester)) => {
                sqlx::query!(
                    "DELETE FROM active_semester WHERE member = ? AND semester = ?",
                    semester_update.member, semester_update.semester
                ).query(conn).await
            }
            (None, None) => {}
        }
    }
}
