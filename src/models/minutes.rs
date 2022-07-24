use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::graphql::guards::Permission;
use crate::models::member::Member;
use crate::models::GqlDate;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Minutes {
    /// The ID of the meeting minutes
    pub id: i64,
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
            let pool: &PgPool = ctx.data_unchecked();
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
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No meeting minutes with id {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, date as \"date: _\", public, private
             FROM minutes WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, date as \"date: _\", public, private
             FROM minutes ORDER BY date DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(name: &str, pool: &PgPool) -> Result<i64> {
        sqlx::query!("INSERT INTO minutes (name) VALUES ($1)", name)
            .execute(pool)
            .await?;

        sqlx::query_scalar!("SELECT id FROM minutes ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, update: UpdatedMeetingMinutes, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = $1, private = $2, public = $3 WHERE id = $4",
            update.name,
            update.private,
            update.public,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct UpdatedMeetingMinutes {
    pub name: String,
    pub public: String,
    pub private: Option<String>,
}
