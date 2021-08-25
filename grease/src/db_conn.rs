use std::sync::Mutex;

use anyhow::Context;
use owning_ref::{MutexGuardRefMut, OwningRefMut};
use sqlx::{Connection, MySql, MySqlConnection, Transaction};

pub struct DbConn {
    transaction: MutexGuardRefMut<MySqlConnection, Transaction<'_, MySql>>,
}

impl DbConn {
    pub async fn connect() -> anyhow::Result<Self> {
        dotenv::dotenv();

        let db_url = std::env::var("DATABASE_URL").context("No database URL provided")?;
        let mut connection = MySqlConnection::connect(db_url).await?;
        let transaction: Transaction<'_, MySql> = connection.begin().await?;
        let transaction = Mutex::new(transaction);

        Ok(Self {
            transaction: OwningRefMut::new(transaction).map_mut(|t| t.lock().unwrap()),
        })
    }

    pub async fn close(self, successful: bool) -> anyhow::Result<()> {
        if successful {
            self.transaction
                .commit()
                .await
                .context("Failed to commit transaction")
        } else {
            self.transaction
                .rollback()
                .await
                .context("Failed to rollback transaction")
        }
    }
}