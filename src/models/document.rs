use async_graphql::{Result, SimpleObject};

use crate::db::DbConn;

/// A link to a Google Doc or other important document.
#[derive(SimpleObject)]
pub struct Document {
    /// The name of the document
    pub name: String,
    /// A link to the document
    pub url: String,
}

impl Document {
    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No document named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs WHERE name = ?", name)
            .fetch_optional(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs ORDER BY name")
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn create(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        if Self::with_name_opt(name, conn).await?.is_some() {
            return Err(format!("A document named {} already exists", name).into());
        }

        sqlx::query!(
            "INSERT INTO google_docs (name, url) VALUES (?, ?)",
            name,
            url
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn set_url(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, conn).await?;

        sqlx::query!("UPDATE google_docs SET url = ? WHERE name = ?", url, name)
            .execute(&mut *conn.get().await)
            .await?;

        Ok(())
    }

    pub async fn delete(name: &str, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, conn).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = ?", name)
            .execute(&mut *conn.get().await)
            .await?;

        Ok(())
    }
}
