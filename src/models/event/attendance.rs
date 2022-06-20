use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use sqlx::MySqlPool;

use crate::models::event::absence_request::{AbsenceRequest, AbsenceRequestState};
use crate::models::event::Event;
use crate::models::member::active_semester::ActiveSemester;
use crate::models::member::Member;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Attendance {
    /// Whether the member is expected to attend the event
    pub should_attend: bool,
    /// Whether the member did attend the event
    pub did_attend: bool,
    /// Whether the member confirmed that they would attend
    pub confirmed: bool,
    /// How late the member was if they attended
    pub minutes_late: i32,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: i32,
}

#[ComplexObject]
impl Attendance {
    /// The email of the member this attendance belongs to
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool: &MySqlPool = ctx.data_unchecked();
        Member::with_email(&self.member, pool).await
    }

    /// The absence request made by the current member, if they requested one
    pub async fn absence_request(&self, ctx: &Context<'_>) -> Result<Option<AbsenceRequest>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        AbsenceRequest::for_member_at_event_opt(&self.member, self.event, pool).await
    }

    /// If the member is not allowed to RSVP, this is why
    pub async fn rsvp_issue(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        let event = Event::with_id(self.event, pool).await?;
        let is_active =
            ActiveSemester::for_member_during_semester(&self.member, &event.semester, pool)
                .await?
                .is_some();

        event.rsvp_issue_for(Some(self), is_active)
    }

    /// Whether the absence is approved
    pub async fn approved_absence(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(absence_request) = self.absence_request(ctx).await? {
            Ok(absence_request.state == AbsenceRequestState::Approved)
        } else {
            Ok(false)
        }
    }

    /// If credit for attending the event should be denied
    pub async fn deny_credit(&self, ctx: &Context<'_>) -> Result<bool> {
        Ok(self.should_attend && !self.did_attend && !self.approved_absence(ctx).await?)
    }
}

impl Attendance {
    pub async fn for_member_at_event(email: &str, event_id: i32, pool: &MySqlPool) -> Result<Self> {
        Self::for_member_at_event_opt(email, event_id, pool)
            .await?
            .ok_or_else(|| {
                format!(
                    "No attendance for member {} at event with id {}",
                    email, event_id
                )
            })
            .map_err(Into::into)
    }

    pub async fn for_member_at_event_opt(
        email: &str,
        event_id: i32,
        pool: &MySqlPool,
    ) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT should_attend as \"should_attend: bool\", did_attend as \"did_attend: bool\",
                 confirmed as \"confirmed: bool\", minutes_late, member, event
             FROM attendance WHERE member = ? && event = ?",
            email,
            event_id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_event(event_id: i32, pool: &MySqlPool) -> Result<Vec<Self>> {
        // TODO: verify_exists
        Event::with_id(event_id, pool).await?;

        sqlx::query_as!(
            Self,
            "SELECT should_attend as \"should_attend: bool\", did_attend as \"did_attend: bool\",
                 confirmed as \"confirmed: bool\", minutes_late, member, event
             FROM attendance WHERE event = ?",
            event_id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create_for_new_member(
        email: &str,
        semester: &str,
        pool: &MySqlPool,
    ) -> Result<()> {
        let events = Event::for_semester(semester, pool).await?;

        // TODO: make batch query
        let now = crate::util::now()?;
        for event in events {
            let should_attend = if event.call_time.0 < now {
                false
            } else {
                event.default_attend
            };
            sqlx::query!(
                "INSERT IGNORE INTO attendance (event, should_attend, member) VALUES (?, ?, ?)",
                event.id,
                should_attend,
                email
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn create_for_new_event(event_id: i32, pool: &MySqlPool) -> Result<()> {
        let event = Event::with_id(event_id, pool).await?;
        let active_members = Member::active_during(&event.semester, pool).await?;

        // TODO: make batch query
        for member in active_members {
            sqlx::query!(
                "INSERT INTO attendance (event, should_attend, member) VALUES (?, ?, ?)",
                event_id,
                event.default_attend,
                member.email
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn excuse_unconfirmed(event_id: i32, pool: &MySqlPool) -> Result<()> {
        Event::with_id(event_id, pool).await?;

        sqlx::query!(
            "UPDATE attendance SET should_attend = false WHERE event = ? AND confirmed = false",
            event_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update(
        event_id: i32,
        email: &str,
        update: AttendanceUpdate,
        pool: &MySqlPool,
    ) -> Result<()> {
        // TODO: verify exists
        Self::for_member_at_event(email, event_id, pool).await?;

        sqlx::query!(
            "UPDATE attendance SET \
            should_attend = ?, did_attend = ?, confirmed = ?, minutes_late = ? \
            WHERE member = ? AND event = ?",
            update.should_attend,
            update.did_attend,
            update.confirmed,
            update.minutes_late,
            email,
            event_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn rsvp_for_event(
        event_id: i32,
        email: &str,
        attending: bool,
        pool: &MySqlPool,
    ) -> Result<()> {
        let event = Event::with_id(event_id, pool).await?;
        let attendance = Self::for_member_at_event_opt(email, event_id, pool).await?;
        let is_active = ActiveSemester::for_member_during_semester(email, &event.semester, pool)
            .await?
            .is_some();
        event.ensure_no_rsvp_issue(attendance.as_ref(), is_active)?;

        sqlx::query!(
            "UPDATE attendance SET should_attend = ?, confirmed = true \
             WHERE event = ? AND member = ?",
            attending,
            event_id,
            email
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn confirm_for_event(event_id: i32, email: &str, pool: &MySqlPool) -> Result<()> {
        // TODO: verify_exists
        Event::with_id(event_id, pool).await?;
        Attendance::for_member_at_event(email, event_id, pool).await?;

        sqlx::query!(
            "UPDATE attendance SET should_attend = true, confirmed = true \
                WHERE event = ? AND member = ?",
            event_id,
            email
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct AttendanceUpdate {
    pub should_attend: bool,
    pub did_attend: bool,
    pub confirmed: bool,
    pub minutes_late: i32,
}
