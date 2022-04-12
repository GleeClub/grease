use async_graphql::{Context, Object, Result};

use crate::db::DbConn;
use crate::models::event::EventType;
use crate::models::member::SectionType;
use crate::models::money::TransactionType;
use crate::models::permissions::{Permission, Role};
use crate::models::song::MediaType;

pub struct StaticData;

#[Object]
impl StaticData {
    pub async fn media_types(&self, ctx: &Context<'_>) -> Result<Vec<MediaType>> {
        let conn = ctx.data_unchecked::<DbConn>();
        MediaType::all(conn).await
    }

    pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Permission::all(conn).await
    }

    pub async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Role::all(conn).await
    }

    pub async fn event_types(&self, ctx: &Context<'_>) -> Result<Vec<EventType>> {
        let conn = ctx.data_unchecked::<DbConn>();
        EventType::all(conn).await
    }

    pub async fn sections(&self, ctx: &Context<'_>) -> Result<Vec<SectionType>> {
        let conn = ctx.data_unchecked::<DbConn>();
        SectionType::all(conn).await
    }

    pub async fn transaction_types(&self, ctx: &Context<'_>) -> Result<Vec<TransactionType>> {
        let conn = ctx.data_unchecked::<DbConn>();
        TransactionType::all(conn).await
    }
}
