use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::models::event::EventType;
use crate::models::member::SectionType;
use crate::models::money::TransactionType;
use crate::models::permissions::{Permission, Role};
use crate::models::song::MediaType;

pub struct StaticData;

/// A collection of static data
#[Object]
impl StaticData {
    /// The types of media available for song links
    pub async fn media_types(&self, ctx: &Context<'_>) -> Result<Vec<MediaType>> {
        let pool: &PgPool = ctx.data_unchecked();
        MediaType::all(pool).await
    }

    /// All permissions used by the site
    pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let pool: &PgPool = ctx.data_unchecked();
        Permission::all(pool).await
    }

    /// All officer roles
    pub async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let pool: &PgPool = ctx.data_unchecked();
        Role::all(pool).await
    }

    /// All types of events
    pub async fn event_types(&self, ctx: &Context<'_>) -> Result<Vec<EventType>> {
        let pool: &PgPool = ctx.data_unchecked();
        EventType::all(pool).await
    }

    /// All voice sections members can sing in
    pub async fn sections(&self, ctx: &Context<'_>) -> Result<Vec<SectionType>> {
        let pool: &PgPool = ctx.data_unchecked();
        SectionType::all(pool).await
    }

    /// All types of transactions
    pub async fn transaction_types(&self, ctx: &Context<'_>) -> Result<Vec<TransactionType>> {
        let pool: &PgPool = ctx.data_unchecked();
        TransactionType::all(pool).await
    }
}
