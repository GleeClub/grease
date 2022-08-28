use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::models::event::Event;
use crate::models::member::Member;
use crate::models::DateTime;

/// A request by a member to not lose credit for missing an event
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct AbsenceRequest {
    /// The reason the member petitioned for absence with
    pub reason: String,
    /// The current state of the request
    pub state: AbsenceRequestStatus,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: i64,
    #[graphql(skip)]
    pub time: OffsetDateTime,
}

#[ComplexObject]
impl AbsenceRequest {
    /// The time this request was placed
    pub async fn time(&self) -> DateTime {
        DateTime::from(self.time.clone())
    }

    /// The event they requested absence from
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Event> {
        let pool: &PgPool = ctx.data_unchecked();
        Event::with_id(self.event, pool).await
    }

    /// The member that requested an absence
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        Member::with_email(&self.member, pool).await
    }
}

/// The current status of an absence request
#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "absence_request_status", rename_all = "snake_case")]
pub enum AbsenceRequestStatus {
    /// The request hasn't been responded to yet
    Pending,
    /// The request has been approved
    Approved,
    /// The request has been denied
    Denied,
}

impl AbsenceRequest {
    pub async fn for_member_at_event(email: &str, event_id: i64, pool: &PgPool) -> Result<Self> {
        Self::for_member_at_event_opt(email, event_id, pool)
            .await?
            .ok_or_else(|| {
                format!(
                    "No absence request for member {} at event with id {}",
                    email, event_id
                )
            })
            .map_err(Into::into)
    }

    pub async fn for_member_at_event_opt(
        email: &str,
        event_id: i64,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT \"time\" as \"time: _\", reason, state as \"state: _\", member, event
             FROM absence_requests WHERE member = $1 AND event = $2",
            email,
            event_id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester_name: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT \"time\" as \"time: _\", reason, state as \"state: _\", member, event
             FROM absence_requests
             WHERE event IN (SELECT id FROM events WHERE semester = $1)
             ORDER BY time DESC",
            semester_name
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn submit(event_id: i64, email: &str, reason: &str, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "INSERT INTO absence_requests (member, event, reason) VALUES ($1, $2, $3)",
            email,
            event_id,
            reason
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_state(
        event_id: i64,
        member: &str,
        state: AbsenceRequestStatus,
        pool: &PgPool,
    ) -> Result<()> {
        AbsenceRequest::for_member_at_event(member, event_id, pool).await?;

        sqlx::query!(
            "UPDATE absence_requests SET state = $1 WHERE event = $2 AND member = $3",
            state as _,
            event_id,
            member
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
