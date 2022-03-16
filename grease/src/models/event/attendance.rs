use async_graphql::ComplexObject;
use crate::db_conn::DbConn;
use chrono::{NaiveDateTime, Local};

#[derive(ComplexObject)]
pub struct Attendance {
    /// Whether the member is expected to attend the event
    pub should_attend: bool,
    /// Whether the member did attend the event
    pub did_attend: bool,
    /// Whether the member confirmed that they would attend
    pub confirmed: bool,
    /// How late the member was if they attended
    pub minutes_late: isize,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: isize,
}


#[complex]
impl Attendance {
    /// The email of the member this attendance belongs to
    pub async fn member(&self) -> async_graphql::Result<Member> {
        Member::load(&self.member).await
    }

    /// The absence request made by the current member, if they requested one
    pub async fn absence_request(&self, ctx: Context<'_>) -> Result<AbsenceRequest> {
        AbsenceRequest::for_member_at_event(&self.member, self.event).await
    }

    /// If the member is not allowed to RSVP, this is why
    pub async fn rsvp_issue(&self, ctx: Context<'_>) -> Result<String> {
        let event = Event::with_id(self.event, ctx.data_unchecked::<DbConn>()).await?;
        event.rsvp_issue_for(self.member).await
    }

    /// Whether the absence is approved
    pub async fn approved_absence(&self, ctx: Context<'_>) -> Result<bool> {
        if let Some(absence_request) = self.absence_request(ctx).await? {
            absence_request.state == AbsenceRequestState::Approved
        } else {
            false
        }
    }

    /// If credit for attending the event should be denied
    pub async fn deny_credit(&self, ctx: Context<'_>) -> bool {
        Ok(self.should_attend && !self.did_attend && !self.approved_absence(ctx).await?)
    }
}

impl Attendance {
    pub async fn for_member_at_event(email: &str, event_id: isize, conn: &DbConn) -> Result<Self> {
        Self::for_member_at_event(email, event_id, conn).await?.ok_or_else(|| anyhow::anyhow!("No attendance for member {} at event with id {}", email, event_id))
    }

    pub async fn for_member_at_event_opt(email: &str, event_id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM attendance WHERE member = ? && event = ?", email, self.event)
            .fetch_optional(conn)
            .await
    }

    pub async fn for_event(event_id: isize, conn: &DbConn) -> Result<Self> {
        Event::verify_exists(event_id, conn).await?;

        sqlx::query_as!(Self, "SELECT * FROM attendance WHERE event = ?", self.event)
            .fetch_all(conn)
            .await
    }

    pub async fn create_for_new_member(email: &str, conn: &DbConn) -> Result<()> {
        let current_semester = Semester::current(conn).await?;
        let events = Event::for_semester(current_semester.name).await?;

        // TODO: make batch query
        let now = NaiveDateTime<Local>::now();

        for event in events {
            let should_attend = if event.call_time < now { false } else { event.default_attend };
            sqlx::query!("INSERT IGNORE INTO attendance (event, should_attend, member) VALUES (?, ?, ?)", event.id, should_attend, email)
                .query(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn create_for_new_event(event_id: isize, conn: &DbConn) -> Result<()> {
        let event = Event::with_id(event_id).await?;
        let active_members = Member::those_active_during(event.semester, conn).await?;

        // TODO: make batch query
        for member in active_members {
            sqlx::query!("INSERT INTO attendance (event, should_attend, member) VALUES (?, ?, ?)", event_id, event.default_attend, member.email)
                .query(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn excuse_unconfirmed(event_id: isize, conn: &DbConn) -> Result<()> {
        let event = Event::with_id(event_id).await?;

        sqlx::query!("UPDATE attendance SET should_attend = false WHERE event = ? AND confirmed = false", event_id)
            .query(conn)
            .await
    }

    pub async fn update(event_id: isize, email: &str, update: AttendanceUpdate, conn: &DbConn) -> Result<()> {
        Self::verify_for_member_at_event(event_id, email, conn).await?;

        sqlx::query!("UPDATE attendance SET \
            should_attend = ?, did_attend = ?, confirmed = ?, minutes_late = ? \
            WHERE member = ? AND event = ?", 
            update.should_attend, update.did_attend, update.confirmed, update.minutes_late,
            email, event_id
            )
            .query(conn)
            .await
    }

    pub async fn rsvp_for_event(event_id: isize, email: &str, attending: bool, conn: &DbConn) -> Result<()> {
        let event = Event::with_id(event_id).await?;
        let attendance = Self::for_member_at_event(email, event_id, conn).await?;
        event.ensure_no_rsvp_issue(member, attendance).await?;

        sqlx::query!("UPDATE attendance SET should_attend = ?, confirmed = true \
        WHERE event = ? AND member = ?", attending, event_id, email
            )
            .query(conn)
            .await
    }

    pub async fn confirm_for_event(event_id: isize, email: &str, conn: &DbConn) -> Result<()> {
        Event::verify_exists(event_id, conn).await?;
        Self::verify_for_member_at_event(email, event_id, conn).await?;

        sqlx::query!(
            "UPDATE attendance SET should_attend = true, confirmed = true \
                WHERE event = ? AND member = ?", event_id, email)
            .query(conn)
            .await
    }
}
