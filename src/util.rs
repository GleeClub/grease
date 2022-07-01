use std::env;

use sqlx::MySqlPool;
use time::OffsetDateTime;

use crate::error::{GreaseError, GreaseResult};

pub fn now() -> GreaseResult<OffsetDateTime> {
    OffsetDateTime::try_now_local().map_err(Into::into)
}

pub async fn connect_to_db() -> GreaseResult<MySqlPool> {
    let db_uri = env::var("DATABASE_URL").map_err(|_e| GreaseError::DbUrlNotProvided)?;

    MySqlPool::connect(&db_uri).await.map_err(Into::into)
}
