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
    pub async fn with_key(key: &str, mut conn: DbConn<'_>) -> Result<Self> {
        Self::load_opt(key, conn)
            .await?
            .ok_or_else(|| format!("No variable with key {}", key))
    }

    pub async fn with_key_opt(key: &str, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM variable WHERE `key` = ?", key)
            .fetch_optional(conn)
            .await
    }

    pub async fn set(key: &str, value: &str, mut conn: DbConn<'_>) -> Result<()> {
        if Self::load_opt(key, conn).await?.is_some() {
            sqlx::query!("UPDATE variable SET value = ? WHERE `key` = ?", value, key)
                .execute(conn)
                .await
        } else {
            sqlx::query!(
                "INSERT INTO variable (`key`, value) VALUES (?, ?)",
                key,
                value
            )
            .execute(conn)
            .await
        }
    }

    pub async fn unset(key: &str, mut conn: DbConn<'_>) -> Result<()> {
        sqlx::query!("DELETE FROM variable WHERE `key` = ?", key)
            .execute(conn)
            .await
    }
}
