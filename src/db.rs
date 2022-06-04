use anyhow::{Context as _, Result};
use async_graphql::Context;
use sqlx::{Connection, MySqlConnection};
use tokio::sync::{Mutex, MutexGuard};

pub struct DbConn(Mutex<MySqlConnection>);

impl DbConn {
    pub async fn connect() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL").context("No database URL provided")?;
        let mut conn = MySqlConnection::connect(&db_url).await?;

        sqlx::query!("START TRANSACTION").execute(&mut conn).await?;

        Ok(Self(Mutex::new(conn)))
    }

    pub fn from_ctx<'c>(ctx: &Context<'c>) -> &'c Self {
        ctx.data_unchecked::<DbConn>()
    }

    pub async fn get(&self) -> MutexGuard<'_, MySqlConnection> {
        self.0.lock().await
    }

    pub fn into_inner(self) -> MySqlConnection {
        self.0.into_inner()
    }

    pub async fn close(&self, successful: bool) -> Result<()> {
        if successful {
            sqlx::query!("COMMIT")
                .execute(&mut *self.get().await)
                .await
                .context("Failed to commit transaction")?;
        } else {
            sqlx::query!("ROLLBACK")
                .execute(&mut *self.get().await)
                .await
                .context("Failed to rollback transaction")?;
        }

        Ok(())
    }
}
