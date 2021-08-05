use anyhow::bail;
use async_graphql::{Object, Result};
use sqlx::FromRow;

use crate::db_conn::DbConn;

/// A link to a Google Doc or other important document.
#[derive(Object, FromRow)]
pub struct Document {
    /// The name of the document
    pub name: String,
    /// A link to the document
    pub url: String,
}

#[Object]
impl Document {
    pub async fn load_all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Document, "SELECT * FROM google_docs ORDER BY name")
            .query_all(&mut *conn)
            .await
            .into()
    }

    pub async fn load_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Document, "SELECT * FROM google_docs WHERE name = ?", name)
            .query_optional(&mut *conn)
            .await
            .into()
    }

    pub async fn load(name: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(name, conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No document named {}", name))
            .into()
    }

    pub async fn create(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        if Self::load_opt(name, conn).await?.is_some() {
            bail!("A document named {} already exists", name);
        }

        sqlx::query!(
            "INSERT INTO google_docs (name, url) VALUES (?, ?)",
            name,
            url
        )
        .query(&mut *conn)
        .await?
        .into()
    }

    pub async fn set_url(name: &str, url: &str, conn: &DbConn) -> Result<()> {
        Self::load(name, conn).await?;

        sqlx::query!("UPDATE google_docs SET url = ? WHERE name = ?", url, name)
            .query(&mut *conn)
            .await
            .into()
    }

    pub async fn delete(name: &str, conn: &DbConn) -> Result<()> {
        Self::load(name, conn).await?;

        sqlx::query!("DELETE FROM google_docs WHERE name = ?", name)
            .query(&mut *conn)
            .await
            .into()
    }
}

/// Arbitrary variables for developer usage.
#[derive(Object, FromRow)]
pub struct Variable {
    /// The name of the variable.
    pub key: String,
    /// The value of the variable.
    pub value: String,
}

#[Object]
impl Variable {
    pub async fn load_opt(key: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM variable WHERE key = ?", key)
            .query_optional(&mut *conn)
            .await
            .into()
    }

    pub async fn load(key: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(key, conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No variable with key {}", key))
            .into()
    }

    pub async fn set(key: &str, value: &str, conn: &DbConn) -> Result<()> {
        if Self::load_opt(key, conn).await?.is_some() {
            sqlx::query!("UPDATE variable SET value = ? WHERE key = ?", value, key)
                .query(&mut *conn)
                .await
                .into()
        } else {
            sqlx::query!(
                "INSERT INTO variable (key, value), VALUES (?, ?)",
                key,
                value
            )
            .query(&mut *conn)
            .await
            .into()
        }
    }

    pub async fn unset(key: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM variable WHERE key = ?", key)
            .query(&mut *conn)
            .await
            .into()
    }
}
