use async_graphql::{Result, SimpleObject; InputObject, ComplexObject, Context};
use chrono::NaiveDateTime;
use crate::models::member::member::Member;
use crate::db_conn::DbConn;
use crate::graphql::permission::Permission;

#[derive(SimpleObject)]
pub struct Minutes {
    /// The ID of the meeting minutes
    pub id: isize,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    pub date: NaiveDateTime,
    /// The public, redacted notes visible by all members
    pub public: Option<String>,

    #[graphql(skip)]
    pub private: Option<String>,
}

#[ComplexObject]
impl Minutes {
    /// The private, complete officer notes
    pub async fn private(&self, ctx: &Context<'_>) -> Option<&str> {
        if let Some(user) = ctx.data_opt::<Member>() {
            if user.able_to(Permission::VIEW_COMPLETE_MINUTES) {
                return Some(self.complete);
            }
        }

        None
    }
}

impl Minutes {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(format!("No meeting minutes with id {}", id))
    }

    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes ORDER BY date")
            .query_all(conn)
            .await
    }

    pub async fn create(name: &str, conn: &DbConn) -> Result<isize> {
        sqlx::query!("INSERT INTO minutes (name) VALUES (?)", name)
            .query(conn)
            .await?;

        sqlx::query!("SELECT id FROM minutes ORDER BY id DESC")
            .query(conn)
            .await
    }

    pub async fn update(id: isize, update: MinutesUpdate, conn: &DbConn) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = ?, private = ?, public = ? WHERE id = ?",
            update.name,
            update.complete,
            update.public,
            id
        )
        .query(conn)
        .await
    }

    pub async fn delete(id: isize, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = ?", id)
            .query(conn)
            .await
    }

    // def email
    //   # TODO: implement
    // end
}

#[derive(InputObject)]
pub struct UpdatedMeetingMinutes {
    pub name: String,
    pub public: String,
    pub private: Option<String>,
}
