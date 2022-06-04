use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::event::Event;
use crate::models::member::Member;
use crate::models::GqlDateTime;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct AbsenceRequest {
    /// The time this request was placed
    pub time: GqlDateTime,
    /// The reason the member petitioned for absence with
    pub reason: String,
    /// The current state of the request
    pub state: AbsenceRequestState,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: i32,
}

#[ComplexObject]
impl AbsenceRequest {
    /// The event they requested absence from
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        Event::with_id(self.event, conn).await
    }

    /// The member that requested an absence
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        Member::with_email(&self.member, conn).await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum AbsenceRequestState {
    Pending,
    Approved,
    Denied,
}

impl AbsenceRequest {
    pub async fn for_member_at_event(email: &str, event_id: i32, conn: &DbConn) -> Result<Self> {
        Self::for_member_at_event_opt(email, event_id, conn)
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
        event_id: i32,
        conn: &DbConn,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT `time` as \"time: _\", reason, state as \"state: _\", member, event
             FROM absence_request WHERE member = ? AND event = ?",
            email,
            event_id
        )
        .fetch_optional(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester_name: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT `time` as \"time: _\", reason, state as \"state: _\", member, event
             FROM absence_request
             WHERE event IN (SELECT id FROM event WHERE semester = ?)
             ORDER BY time",
            semester_name
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn submit(event_id: i32, email: &str, reason: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!(
            "INSERT INTO absence_request (member, event, reason) VALUES (?, ?, ?)",
            email,
            event_id,
            reason
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn set_state(
        event_id: i32,
        member: &str,
        state: AbsenceRequestState,
        conn: &DbConn,
    ) -> Result<()> {
        AbsenceRequest::for_member_at_event(member, event_id, conn).await?;

        sqlx::query!(
            "UPDATE absence_request SET state = ? WHERE event = ? AND member = ?",
            state,
            event_id,
            member
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }
}
