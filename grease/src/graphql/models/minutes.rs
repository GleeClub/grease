use async_graphql::{ComplexObject, Context, Object};
use chrono::NaiveDateTime;
use sqlx::{FromRow, MySqlPool, Result};

#[derive(FromRow, ComplexObject)]
pub struct Minutes {
    /// The id of the meeting minutes
    pub id: i32,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    pub date: NaiveDateTime,
    /// The public, redacted notes visible by all members
    pub public: Option<String>,
    #[graphql(skip)]
    pub private: Option<String>,
}

impl Minutes {
    // pub const TABLE_NAME: &str = "minutes";

    pub async fn with_id_opt(id: i32, pool: &MySqlPool) -> Result<Option<Self>> {
        sqlx::query_as!(Minutes, "SELECT * FROM minutes WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn with_id(id: i32, pool: &MySqlPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await
            .and_then(|res| res.ok_or_else(|| format!("No meeting minutes with id {}", id)))
    }

    pub async fn all(pool: &MySqlPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM minutes ORDER BY date")
            .query_all(pool)
            .await
            .into()
    }

    pub async fn create(name: &str, pool: &MySqlPool) -> Result<i32> {
        pool.begin(|tx| {
            sqlx::query!("INSERT INTO minutes (name) VALUES (?)", name)
                .query(tx)
                .await?;
            sqlx::query!("SELECT id FROM minutes ORDER BY id DESC")
                .query(tx)
                .await
                .into()
        })
    }

    pub async fn update(id: i32, form: MinutesUpdate, pool: &MySqlPool) -> Result<()> {
        sqlx::query!(
            "UPDATE minutes SET name = ?, private = ?, public = ? WHERE ID = ?",
            form.name,
            form.complete,
            form.public,
            id
        )
        .query(pool)
        .await
        .into()
    }

    pub async fn delete(id: i32, pool: &MySqlPool) -> Result<()> {
        sqlx::query!("DELETE FROM minutes WHERE id = ?", id)
            .query(pool)
            .await
            .into()
    }

    // def email
    //   # TODO: implement
    // end
}

#[Complex]
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
