use async_graphql::{InputObject, Result, SimpleObject};

use crate::db::DbConn;
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
    pub gig_requirement: i32,
    /// Whether this is the current semester
    pub current: bool,
}

impl Semester {
    pub async fn get_current(conn: &DbConn) -> Result<Self> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester WHERE current = true"
        )
        .fetch_optional(&mut *conn.get().await)
        .await?
        .ok_or_else(|| "No current semester set".into())
    }

    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No semester named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester WHERE name = ?",
            name
        )
        .fetch_optional(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, start_date as \"start_date: _\", end_date as \"end_date: _\",
                 gig_requirement, current as \"current: bool\"
             FROM semester ORDER BY start_date"
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn create(new_semester: NewSemester, conn: &DbConn) -> Result<()> {
        if Self::with_name_opt(&new_semester.name, conn)
            .await?
            .is_some()
        {
            return Err(format!("A semester already exists named {}", new_semester.name).into());
        }

        sqlx::query!(
            "INSERT INTO semester (name, start_date, end_date, gig_requirement)
             VALUES (?, ?, ?, ?)",
            new_semester.name,
            new_semester.start_date,
            new_semester.end_date,
            new_semester.gig_requirement
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn update(name: &str, update: NewSemester, conn: &DbConn) -> Result<()> {
        // check that semester exists
        Self::with_name(name, conn).await?;

        if name != &update.name && Self::with_name_opt(&update.name, conn).await?.is_some() {
            return Err(format!("Another semester is already named {}", update.name).into());
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
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn set_current(name: &str, conn: &DbConn) -> Result<()> {
        if sqlx::query!("SELECT name FROM semester WHERE name = ?", name)
            .fetch_optional(&mut *conn.get().await)
            .await?
            .is_none()
        {
            return Err(format!("No semester named {}", name).into());
        }

        sqlx::query!("UPDATE semester SET current = false")
            .execute(&mut *conn.get().await)
            .await?;
        sqlx::query!("UPDATE semester SET current = true WHERE name = ?", name)
            .execute(&mut *conn.get().await)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewSemester {
    pub name: String,
    pub start_date: GqlDate,
    pub end_date: GqlDate,
    pub gig_requirement: i32,
}
