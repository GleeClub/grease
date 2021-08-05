use async_graphql::{Object, Result};
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::db_conn::DbConn;
use crate::graphql::permission::Permission;

#[derive(FromRow, Object)]
pub struct Minutes {
    /// The ID of the meeting minutes
    pub id: i32,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    pub date: NaiveDateTime,
    // /// The private, complete officer notes
    #[graphql(guard(Permission::VIEW_COMPLETE_MINUTES))]
    pub private: Option<String>,
    /// The public, redacted notes visible by all members
    pub public: Option<String>,
}

impl Minutes {
    pub async fn with_id_opt(id: i32, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes WHERE id = ?", id)
            .fetch_optional(&mut *conn)
            .await
    }

    pub async fn with_id(id: i21, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, &mut *conn)
            .await
            .and_then(|res| res.ok_or_else(format!("No meeting minutes with id {}", id)))
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes ORDER BY date")
            .query_all(&mut *conn)
            .await
            .into()
    }

    pub async fn create(name: &str, conn: &DbConn) -> Result<i32> {
        &mut *conn.begin(|tx| {
            sqlx::query!("INSERT INTO minutes (name) VALUES (?)", name)
                .query(tx)
                .await?;
            let id = sqlx::query!("SELECT id FROM minutes ORDER BY id DESC")
                .query(tx)
                .await?;

            tx.commit().await?;

            Ok(id)
        })
    }

    pub async fn update(id: i32, update: MinutesUpdate, conn: &DbConn) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = ?, private = ?, public = ? WHERE id = ?",
            update.name,
            update.complete,
            update.public,
            id
        )
        .query(&mut *conn)
        .await
        .into()
    }

    pub async fn delete(id: i32, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = ?", id)
            .query(&mut *conn)
            .await
            .into()
    }

    // def email
    //   # TODO: implement
    // end
}
