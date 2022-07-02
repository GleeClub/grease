use async_graphql::{Result, SimpleObject};
use sqlx::PgPool;

/// A link to a Google Doc or other important document.
#[derive(SimpleObject)]
pub struct DocumentLink {
    /// The name of the link
    pub name: String,
    /// The link itself
    pub url: String,
}

impl DocumentLink {
    pub async fn with_name(name: &str, pool: &PgPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No document named {}", name).into())
    }

    pub async fn with_name_opt(name: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs WHERE name = $1", name)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM google_docs ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(name: &str, url: &str, pool: &PgPool) -> Result<()> {
        if Self::with_name_opt(name, pool).await?.is_some() {
            return Err(format!("A document named {} already exists", name).into());
        }

        sqlx::query!(
            "INSERT INTO google_docs (name, url) VALUES ($1, $2)",
            name,
            url
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_url(name: &str, url: &str, pool: &PgPool) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, pool).await?;

        sqlx::query!("UPDATE google_docs SET url = $1 WHERE name = $2", url, name)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete(name: &str, pool: &PgPool) -> Result<()> {
        // TODO: verify exists
        Self::with_name(name, pool).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = $1", name)
            .execute(pool)
            .await?;

        Ok(())
    }
}
