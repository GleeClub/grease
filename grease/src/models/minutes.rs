use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use crate::db::DbConn;
use crate::models::GqlDateTime;
use crate::graphql::permission::Permission;
use crate::models::member::Member;

#[derive(SimpleObject)]
pub struct Minutes {
    /// The ID of the meeting minutes
    pub id: i32,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    pub date: GqlDateTime,
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
    pub async fn with_id(id: i32, mut conn: DbConn<'_>) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(format!("No meeting minutes with id {}", id))
    }

    pub async fn with_id_opt(id: i32, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn all(mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes ORDER BY date")
            .fetch_all(conn)
            .await
    }

    pub async fn create(name: &str, mut conn: DbConn<'_>) -> Result<i32> {
        sqlx::query!("INSERT INTO minutes (name) VALUES (?)", name)
            .execute(conn)
            .await?;

        sqlx::query!("SELECT id FROM minutes ORDER BY id DESC")
            .execute(conn)
            .await
    }

    pub async fn update(id: i32, update: UpdatedMeetingMinutes, mut conn: DbConn<'_>) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = ?, private = ?, public = ? WHERE id = ?",
            update.name,
            update.complete,
            update.public,
            id
        )
        .execute(conn)
        .await
    }

    pub async fn delete(id: i32, mut conn: DbConn<'_>) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = ?", id)
            .execute(conn)
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
