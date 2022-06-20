use async_graphql::{Result, SimpleObject};
use sqlx::MySqlPool;

/// A link to a Google Doc or other important document.
#[derive(SimpleObject)]
pub struct DocumentLink {
    /// The name of the link
    pub name: String,
    /// The link itself
    pub url: String,
}

impl DocumentLink {
    pub async fn with_name(name: &str, pool: &MySqlPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No document named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, pool: &MySqlPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs WHERE name = ?", name)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn all(pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(name: &str, url: &str, pool: &MySqlPool) -> Result<()> {
        if Self::with_name_opt(name, pool).await?.is_some() {
            return Err(format!("A document named {} already exists", name).into());
        }

        sqlx::query!(
            "INSERT INTO google_docs (name, url) VALUES (?, ?)",
            name,
            url
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_url(name: &str, url: &str, pool: &MySqlPool) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, pool).await?;

        sqlx::query!("UPDATE google_docs SET url = ? WHERE name = ?", url, name)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete(name: &str, pool: &MySqlPool) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, pool).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = ?", name)
            .execute(pool)
            .await?;

        Ok(())
    }
}
