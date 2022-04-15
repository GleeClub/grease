use std::sync::{Mutex, MutexGuard};

use anyhow::{Context as _, Result};
use async_graphql::Context;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::pin_mut;
use sqlx::database::HasStatement;
use sqlx::{Connection, Database, Describe, Error, Execute, Executor, MySql, MySqlConnection};

#[derive(Debug)]
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

    pub async fn close(&self, successful: bool) -> Result<()> {
        if successful {
            sqlx::query!("COMMIT")
                .execute(self)
                .await
                .context("Failed to commit transaction")?;
        } else {
            sqlx::query!("ROLLBACK")
                .execute(self)
                .await
                .context("Failed to rollback transaction")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DbConnHandle<'c>(MutexGuard<'c, MySqlConnection>);

impl<'c> Executor<'c> for DbConnHandle<'c> {
    type Database = MySql;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<
            Either<<Self::Database as Database>::QueryResult, <Self::Database as Database>::Row>,
            Error,
        >,
    >
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let mut conn = self.0.lock().unwrap();

        Box::pin(conn.fetch_many(query))

        //     futures_util::stream::Map:(async move {
        //         let s = conn.fetch_many(query).await.await?;
        //         pin_mut!(s);

        //         while let Some(v) = s.try_next().await? {
        //             let _ = futures_util::sink::SinkExt::send(&mut sender, Ok(v)).await;
        //         }

        //         Ok(())
        //     })
        // )
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        self.0.lock().unwrap().fetch_optional(query)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as HasStatement<'q>>::Statement, Error>>
    where
        'c: 'e,
    {
        self.0.lock().unwrap().prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        self.0.lock().unwrap().describe(sql)
    }
}
