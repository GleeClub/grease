use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::models::grades::Grades;

#[derive(SimpleObject)]
#[graphql(complex)]
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
        let pool: &PgPool = ctx.data_unchecked();
        Grades::for_member(&self.member, &self.semester, pool).await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "enrollment", rename_all = "snake_case")]
pub enum Enrollment {
    Class,
    Club,
}

impl ActiveSemester {
    pub async fn all_for_member(member: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT a.member, a.semester, a.enrollment as \"enrollment: _\", a.section
             FROM active_semester a
             JOIN semester s ON a.semester = s.name
             WHERE member = $1
             ORDER BY s.start_date",
            member
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_member_during_semester(
        member: &str,
        semester: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT member, semester, enrollment as \"enrollment: _\", section
             FROM active_semester WHERE member = $1 AND semester = $2",
            member,
            semester
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create_for_member(new_semester: NewActiveSemester, pool: &PgPool) -> Result<()> {
        if Self::for_member_during_semester(&new_semester.member, &new_semester.semester, pool)
            .await?
            .is_some()
        {
            return Err("Member is already active for the current semester".into());
        }

        // TODO: create attendance or something?

        sqlx::query!(
            "INSERT INTO active_semester (member, semester, enrollment, section) VALUES ($1, $2, $3, $4)",
            new_semester.member, new_semester.semester, new_semester.enrollment as _, new_semester.section
        ).execute(pool).await?;

        Ok(())
    }

    pub async fn update(update: NewActiveSemester, pool: &PgPool) -> Result<()> {
        let active_semester =
            Self::for_member_during_semester(&update.member, &update.semester, pool).await?;

        match (update.enrollment, active_semester) {
            (Some(enrollment), Some(_active_semester)) => {
                sqlx::query!(
                    "UPDATE active_semester SET enrollment = $1, section = $2 WHERE member = $3 AND semester = $4",
                    enrollment as _, update.section, update.member, update.semester
                ).execute(pool).await?;
            }
            (Some(enrollment), None) => {
                sqlx::query!(
                    "INSERT INTO active_semester (member, semester, enrollment, section) VALUES ($1, $2, $3, $4)",
                    update.member, update.semester, enrollment as _, update.section
                ).execute(pool).await?;
            }
            (None, Some(_active_semester)) => {
                sqlx::query!(
                    "DELETE FROM active_semester WHERE member = $1 AND semester = $2",
                    update.member,
                    update.semester
                )
                .execute(pool)
                .await?;
            }
            (None, None) => {}
        }

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewActiveSemester {
    pub member: String,
    pub semester: String,
    pub enrollment: Option<Enrollment>,
    pub section: Option<String>,
}
