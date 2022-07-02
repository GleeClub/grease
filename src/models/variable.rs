use async_graphql::{Result, SimpleObject};
use sqlx::{FromRow, PgPool};

/// Arbitrary variables for developer usage.
#[derive(SimpleObject, FromRow)]
pub struct Variable {
    /// The name of the variable.
    pub key: String,
    /// The value of the variable.
    pub value: String,
}

impl Variable {
    pub async fn with_key(key: &str, pool: &PgPool) -> Result<Self> {
        Self::with_key_opt(key, pool)
            .await?
            .ok_or_else(|| format!("No variable with key {}", key))
            .map_err(Into::into)
    }

    pub async fn with_key_opt(key: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM variable WHERE key = $1", key)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn set(key: &str, value: &str, pool: &PgPool) -> Result<()> {
        if Self::with_key_opt(key, pool).await?.is_some() {
            sqlx::query!("UPDATE variable SET value = $1 WHERE key = $2", value, key)
                .execute(pool)
                .await?;
        } else {
            sqlx::query!(
                "INSERT INTO variable (key, value) VALUES ($1, $2)",
                key,
                value
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn unset(key: &str, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM variable WHERE key = $1", key)
            .execute(pool)
            .await?;

        Ok(())
    }
}
