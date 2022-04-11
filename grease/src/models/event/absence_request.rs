use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject};
use time::OffsetDateTime;

use crate::db_conn::DbConn;
use crate::models::event::Event;
use crate::models::member::Member;

#[derive(SimpleObject)]
pub struct AbsenceRequest {
    /// The time this request was placed
    pub time: OffsetDateTime,
    /// The reason the member petitioned for absence with
    pub reason: String,
    /// The current state of the request
    pub state: AbsenceRequestState,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: i64,
}

#[ComplexObject]
impl AbsenceRequest {
    /// The event they requested absence from
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Event> {
        let conn = ctx.data_unchecked::<DbConn>();
        Event::with_id(self.event, conn).await
    }

    /// The member that requested an absence
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let conn = ctx.data_unchecked::<DbConn>();
        Member::with_email(&self.member, conn).await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum AbsenceRequestState {
    Pending,
    Approved,
    Denied,
}

impl AbsenceRequest {
    pub async fn for_member_at_event(
        &self,
        email: &str,
        event_id: i64,
        conn: &DbConn<'_>,
    ) -> Result<Self> {
        Self::for_member_at_event_opt(email, event_id, conn)
            .await?
            .ok_or_else(|| {
                format!(
                    "No absence request for member {} at event with id {}",
                    email, event_id
                )
            })
    }

    pub async fn for_member_at_event_opt(
        &self,
        email: &str,
        event_id: i64,
        conn: &DbConn<'_>,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM absence_request WHERE member = ? AND event = ?",
            email,
            event_id
        )
        .query_optional(conn)
        .await
    }

    pub async fn for_semester(&self, semester_name: &str, conn: &DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM absence_request WHERE event IN
             (SELECT id FROM event WHERE semester = ?)
             ORDER BY time",
            semester_name
        )
        .query_all(conn)
        .await
    }

    pub async fn submit(
        event_id: i64,
        email: &str,
        reason: &str,
        conn: &DbConn<'_>,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO absence_request (member, event, reason) VALUES (?, ?, ?)",
            email,
            event_id,
            reason
        )
        .query(conn)
        .await
    }

    pub async fn set_state(
        event_id: i64,
        member: &str,
        state: AbsenceRequestState,
        conn: &DbConn<'_>,
    ) -> Result<()> {
        AbsenceRequest::for_member_at_event(member, event_id, conn).await?;

        sqlx::query!(
            "UPDATE absence_request SET state = ? WHERE event = ? AND member = ?",
            state,
            event_id,
            member
        )
        .query(conn)
        .await
    }
}
