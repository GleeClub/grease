use async_graphql::{InputObject, Result, SimpleObject};
use time::{Date, OffsetDateTime};

use crate::db_conn::DbConn;

#[derive(SimpleObject)]
pub struct Semester {
    /// The name of the semester
    pub name: String,
    /// When the semester starts
    pub start_date: OffsetDateTime,
    /// When the semester ends
    pub end_date: OffsetDateTime,
    /// How many volunteer gigs are required for the semester (default: 5)
    pub gig_requirement: i64,
    /// Whether this is the current semester
    pub current: bool,
}

impl Semester {
    pub async fn get_current(conn: &DbConn<'_>) -> Result<Self> {
        sqlx::query_as!(Self, "SELECT * FROM semester WHERE current = true")
            .fetch_optional(conn)
            .await?
            .ok_or_else(|| "No current semester set".to_owned())
    }

    pub async fn with_name(name: &str, conn: &DbConn<'_>) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No semester named {}", name))
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM semester WHERE name = ?", name)
            .fetch_optional(conn)
            .await
    }

    pub async fn all(conn: &DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM semester ORDER BY start_date")
            .fetch_all(conn)
            .await
    }

    pub async fn create(new_semester: NewSemester, conn: &DbConn<'_>) -> Result<()> {
        if Self::with_name_opt(&new_semester.name, conn)
            .await?
            .is_some()
        {
            return Err(format!(
                "A semester already exists named {}",
                new_semester.name
            ));
        }

        sqlx::query!(
            "INSERT INTO semester (name, start_date, end_date, gig_requirement)
             VALUES (?, ?, ?, ?)",
            new_semester.name,
            new_semester.start_date,
            new_semester.end_date,
            new_semester.gig_requirement
        )
        .execute(conn)
        .await
    }

    pub async fn update(name: &str, update: NewSemester, conn: &DbConn<'_>) -> Result<()> {
        // check that semester exists
        Self::with_name(name, conn).await?;

        if name != &update.name && Self::with_name_opt(&update.name, conn).await?.is_some() {
            return Err(format!("Another semester is already named {}", update.name));
        }

        sqlx::query!(
            "UPDATE semester SET
             name = ?, start_date = ?, end_date = ?, gig_requirement = ?
             WHERE name = ?",
            update.name,
            update.start_date,
            update.end_date,
            update.gig_requirement,
            name
        )
        .query(conn)
        .await
        .into()
    }

    pub async fn set_current(name: &str, conn: &DbConn<'_>) -> Result<()> {
        if sqlx::query!("SELECT name FROM semester WHERE name = ?", name)
            .query_optional(conn)
            .await?
            .is_none()
        {
            return Err(format!("No semester named {}", name));
        }

        sqlx::query!("UPDATE semester SET current = false")
            .query(conn)
            .await?;
        sqlx::query!("UPDATE semester SET current = true WHERE name = ?", true)
            .query(conn)
            .await?;
    }
}

#[derive(InputObject)]
pub struct NewSemester {
    pub name: String,
    pub start_date: Date,
    pub end_date: Date,
    pub gig_requirement: i64,
}
