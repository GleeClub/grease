use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use sqlx::MySqlPool;

use crate::graphql::guards::Permission;
use crate::models::member::Member;
use crate::models::GqlDate;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Minutes {
    /// The ID of the meeting minutes
    pub id: i32,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    pub date: GqlDate,
    /// The public, redacted notes visible by all members
    pub public: Option<String>,

    #[graphql(skip)]
    pub private: Option<String>,
}

#[ComplexObject]
impl Minutes {
    /// The private, complete officer notes
    pub async fn private(&self, ctx: &Context<'_>) -> Result<Option<&String>> {
        if let Some(user) = ctx.data_opt::<Member>() {
            let pool: &MySqlPool = ctx.data_unchecked();
            if Permission::VIEW_COMPLETE_MINUTES
                .granted_to(&user.email, pool)
                .await?
            {
                return Ok(self.private.as_ref());
            }
        }

        Ok(None)
    }
}

impl Minutes {
    pub async fn with_id(id: i32, pool: &MySqlPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No meeting minutes with id {}", id).into())
    }

    pub async fn with_id_opt(id: i32, pool: &MySqlPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, date as \"date: _\", public, private
             FROM minutes WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, date as \"date: _\", public, private
             FROM minutes ORDER BY date"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(name: &str, pool: &MySqlPool) -> Result<i32> {
        sqlx::query!("INSERT INTO minutes (name) VALUES (?)", name)
            .execute(pool)
            .await?;

        sqlx::query_scalar!("SELECT id FROM minutes ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i32, update: UpdatedMeetingMinutes, pool: &MySqlPool) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = ?, private = ?, public = ? WHERE id = ?",
            update.name,
            update.private,
            update.public,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(id: i32, pool: &MySqlPool) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = ?", id)
            .execute(pool)
            .await?;

        Ok(())
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
