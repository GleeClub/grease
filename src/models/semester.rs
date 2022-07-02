use async_graphql::{InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::models::{GqlDate, GqlDateTime};

#[derive(SimpleObject)]
pub struct Semester {
    /// The name of the semester
    pub name: String,
    /// When the semester starts
    pub start_date: GqlDateTime,
    /// When the semester ends
    pub end_date: GqlDateTime,
    /// How many volunteer gigs are required for the semester (default: 5)
    pub gig_requirement: i64,
    /// Whether this is the current semester
    pub current: bool,
}

impl Semester {
    pub async fn get_current(pool: &PgPool) -> Result<Self> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester WHERE current = true"
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| "No current semester set".into())
    }

    pub async fn with_name(name: &str, pool: &PgPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No semester named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester WHERE name = $1",
            name
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester ORDER BY start_date"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(new_semester: NewSemester, pool: &PgPool) -> Result<()> {
        if Self::with_name_opt(&new_semester.name, pool)
            .await?
            .is_some()
        {
            return Err(format!("A semester already exists named {}", new_semester.name).into());
        }

        sqlx::query!(
            "INSERT INTO semester (name, start_date, end_date, gig_requirement)
             VALUES ($1, $2, $3, $4)",
            new_semester.name,
            new_semester.start_date.0,
            new_semester.end_date.0,
            new_semester.gig_requirement
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update(name: &str, update: NewSemester, pool: &PgPool) -> Result<()> {
        // check that semester exists
        Self::with_name(name, pool).await?;

        if name != &update.name && Self::with_name_opt(&update.name, pool).await?.is_some() {
            return Err(format!("Another semester is already named {}", update.name).into());
        }

        sqlx::query!(
            "UPDATE semester SET
             name = $1, start_date = $2, end_date = $3, gig_requirement = $4
             WHERE name = $5",
            update.name,
            update.start_date.0,
            update.end_date.0,
            update.gig_requirement,
            name
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_current(name: &str, pool: &PgPool) -> Result<()> {
        if sqlx::query!("SELECT name FROM semester WHERE name = $1", name)
            .fetch_optional(pool)
            .await?
            .is_none()
        {
            return Err(format!("No semester named {}", name).into());
        }

        sqlx::query!("UPDATE semester SET current = false")
            .execute(pool)
            .await?;
        sqlx::query!("UPDATE semester SET current = true WHERE name = $1", name)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewSemester {
    pub name: String,
    pub start_date: GqlDate,
    pub end_date: GqlDate,
    pub gig_requirement: i64,
}
