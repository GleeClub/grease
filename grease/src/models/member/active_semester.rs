use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::grades::Grades;

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
        let mut conn = get_conn(ctx);
        Grades::for_member(&self.member, &self.semester, &mut conn).await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Enrollment {
    Class,
    Club,
}

impl ActiveSemester {
    pub async fn all_for_member(member: &str, mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT a.* FROM active_semester a
             JOIN semester s ON a.semester = s.name
             WHERE member = ?
             ORDER BY s.start_date",
            member
        )
        .fetch_all(conn)
        .await
    }

    pub async fn for_member_during_semester(
        member: &str,
        semester: &str,
        mut conn: DbConn<'_>,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM active_semester WHERE member = ? AND semester = ?",
            member,
            semester
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn create_for_member(
        new_semester: NewActiveSemester,
        mut conn: DbConn<'_>,
    ) -> Result<()> {
        if Self::for_member_during_semester(&new_semester.member, &new_semester.semester, conn)
            .await?
            .is_some()
        {
            return Err("Member is already active for the current semester".to_owned());
        }

        // TODO: create attendance or something?

        sqlx::query!(
            "INSERT INTO active_semester (member, semester, enrollment, section) VALUES (?, ?, ?, ?)",
            new_semester.member, new_semester.semester, new_semester.enrollment, new_semester.section
        ).execute(conn).await
    }

    pub async fn update(update: NewActiveSemester, mut conn: DbConn<'_>) -> Result<()> {
        let active_semester = Self::for_semester(&update.member, &update.semester, conn).await?;

        match (update.enrollment, active_semester) {
            (Some(enrollment), Some(active_semester)) => {
                sqlx::query!(
                    "UPDATE active_semester SET enrollment = ?, section = ? WHERE member = ? AND semester = ?",
                    enrollment, update.section, update.member, update.semester
                ).execute(conn).await
            }
            (Some(enrollment), None) => {
                sqlx::query!(
                    "INSERT INTO active_semester (member, semester, enrollment, section) VALUES (?, ?, ?, ?)",
                    update.member, update.semester, enrollment, update.section
                ).execute(conn).await
            }
            (None, Some(active_semester)) => {
                sqlx::query!(
                    "DELETE FROM active_semester WHERE member = ? AND semester = ?",
                    update.member, update.semester
                ).execute(conn).await
            }
            (None, None) => Ok(())
        }
    }
}

#[derive(InputObject)]
pub struct NewActiveSemester {
    pub member: String,
    pub semester: String,
    pub enrollment: Option<Enrollment>,
    pub section: Option<String>,
}
