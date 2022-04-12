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
        let mut conn = get_conn(ctx);
        MediaType::all(conn).await
    }

    pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let mut conn = get_conn(ctx);
        Permission::all(conn).await
    }

    pub async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let mut conn = get_conn(ctx);
        Role::all(conn).await
    }

    pub async fn event_types(&self, ctx: &Context<'_>) -> Result<Vec<EventType>> {
        let mut conn = get_conn(ctx);
        EventType::all(conn).await
    }

    pub async fn sections(&self, ctx: &Context<'_>) -> Result<Vec<SectionType>> {
        let mut conn = get_conn(ctx);
        SectionType::all(conn).await
    }

    pub async fn transaction_types(&self, ctx: &Context<'_>) -> Result<Vec<TransactionType>> {
        let mut conn = get_conn(ctx);
        TransactionType::all(conn).await
    }
}
