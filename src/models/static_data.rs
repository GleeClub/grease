use async_graphql::{Context, Object, Result};
use sqlx::MySqlPool;

use crate::models::event::EventType;
use crate::models::member::SectionType;
use crate::models::money::TransactionType;
use crate::models::permissions::{Permission, Role};
use crate::models::song::MediaType;

pub struct StaticData;

#[Object]
impl StaticData {
    pub async fn media_types(&self, ctx: &Context<'_>) -> Result<Vec<MediaType>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        MediaType::all(pool).await
    }

    pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        Permission::all(pool).await
    }

    pub async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        Role::all(pool).await
    }

    pub async fn event_types(&self, ctx: &Context<'_>) -> Result<Vec<EventType>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        EventType::all(pool).await
    }

    pub async fn sections(&self, ctx: &Context<'_>) -> Result<Vec<SectionType>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        SectionType::all(pool).await
    }

    pub async fn transaction_types(&self, ctx: &Context<'_>) -> Result<Vec<TransactionType>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        TransactionType::all(pool).await
    }
}
