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
    pub async fn with_name(name: &str, conn: DbConn<'_>) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No document named {}", name))
            .into()
    }

    pub async fn with_name_opt(name: &str, conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs WHERE name = ?", name)
            .fetch_optional(conn)
            .await
            .into()
    }

    pub async fn all(conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs ORDER BY name")
            .fetch_all(*conn)
            .await
            .into()
    }

    pub async fn create(name: &str, url: &str, conn: DbConn<'_>) -> Result<()> {
        if Self::with_name_opt(name, conn).await?.is_some() {
            return Err(format!("A document named {} already exists", name).into());
        }

        sqlx::query!(
            "INSERT INTO google_docs (name, url) VALUES (?, ?)",
            name,
            url
        )
        .execute(*conn)
        .await
        .into()
    }

    pub async fn set_url(name: &str, url: &str, conn: DbConn<'_>) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, conn).await?;

        sqlx::query!("UPDATE google_docs SET url = ? WHERE name = ?", url, name)
            .execute(*conn)
            .await
            .into()
    }

    pub async fn delete(name: &str, conn: DbConn<'_>) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, conn).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = ?", name)
            .execute(*conn)
            .await
            .into()
    }
}
