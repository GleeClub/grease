use async_graphql::{Result, SimpleObject};
use sqlx::FromRow;

use crate::db::DbConn;

/// Arbitrary variables for developer usage.
#[derive(SimpleObject, FromRow)]
pub struct Variable {
    /// The name of the variable.
    pub key: String,
    /// The value of the variable.
    pub value: String,
}

impl Variable {
    pub async fn with_key(key: &str, conn: &DbConn) -> Result<Self> {
        Self::with_key_opt(key, conn)
            .await?
            .ok_or_else(|| format!("No variable with key {}", key))
            .map_err(Into::into)
    }

    pub async fn with_key_opt(key: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM variable WHERE `key` = ?", key)
            .fetch_optional(conn)
            .await
            .map_err(Into::into)
    }

    pub async fn set(key: &str, value: &str, conn: &DbConn) -> Result<()> {
        if Self::with_key_opt(key, conn).await?.is_some() {
            sqlx::query!("UPDATE variable SET value = ? WHERE `key` = ?", value, key)
                .execute(conn)
                .await?;
        } else {
            sqlx::query!(
                "INSERT INTO variable (`key`, value) VALUES (?, ?)",
                key,
                value
            )
            .execute(conn)
            .await?;
        }

        Ok(())
    }

    pub async fn unset(key: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM variable WHERE `key` = ?", key)
            .execute(conn)
            .await?;

        Ok(())
    }
}
