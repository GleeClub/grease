use async_graphql::{SimpleObject, Result};
use crate::db_conn::DbConn;

/// A link to a Google Doc or other important document.
#[derive(SimplObject)]
pub struct Document {
    /// The name of the document
    pub name: String,
    /// A link to the document
    pub url: String,
}

impl Document {
    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No document named {}", name))
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs WHERE name = ?", name)
            .query_optional(conn)
            .await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs ORDER BY name")
            .query_all(conn)
            .await
            .into()
    }

    pub async fn create(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        if Self::load_opt(name, conn).await?.is_some() {
            return Err(format!("A document named {} already exists", name));
        }

        sqlx::query!("INSERT INTO google_docs (name, url) VALUES (?, ?)", name, url)
            .query(conn)
            .await
    }

    pub async fn set_url(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        Self::load(name, conn).await?;

        sqlx::query!("UPDATE google_docs SET url = ? WHERE name = ?", url, name)
            .query(conn)
            .await
    }

    pub async fn delete(name: &str, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        Self::load(name, conn).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = ?", name)
            .query(conn)
            .await
    }
}
