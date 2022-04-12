use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use time::{OffsetDateTime, Duration};
use crate::util::now;
use crate::models::GqlDateTime;
use crate::db::DbConn;
use crate::models::event::attendance::Attendance;
use crate::models::event::carpool::Carpool;
use crate::models::event::gig::{Gig, GigRequest, GigRequestStatus, NewGig};
use crate::models::member::Member;
use crate::models::semester::Semester;
use crate::models::song::Song;

pub mod absence_request;
pub mod attendance;
pub mod carpool;
pub mod gig;
pub mod public;
pub mod uniform;

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Period {
    No,
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Yearly,
}

#[derive(SimpleObject)]
pub struct EventType {
    /// The name of the type of event
    pub name: String,
    /// The amount of points this event is normally worth
    pub weight: i32,
}

impl EventType {
    pub const REHEARSAL: &'static str = "Rehearsal";
    pub const SECTIONAL: &'static str = "Sectional";
    pub const VOLUNTEER_GIG: &'static str = "Volunteer Gig";
    pub const TUTTI_GIG: &'static str = "Tutti Gig";
    pub const OMBUDS: &'static str = "Ombuds";
    pub const OTHER: &'static str = "Other";

    pub async fn all(mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM event_type ORDER BY name")
            .fetch_all(&mut **conn)
            .await
            .into()
    }
}

#[derive(SimpleObject)]
pub struct Event {
    /// The ID of the event
    pub id: i32,
    /// The name of the event
    pub name: String,
    /// The name of the semester this event belongs to
    pub semester: String,
    /// The type of the event (see EventType)
    pub r#type: String,
    /// When members are expected to arrive to the event
    pub call_time: GqlDateTime,
    /// When members are probably going to be released
    pub release_time: Option<GqlDateTime>,
    /// How many points attendance of this event is worth
    pub points: i32,
    /// General information or details about this event
    pub comments: Option<String>,
    /// Where this event will be held
    pub location: Option<String>,
    /// Whether this event counts toward the volunteer gig count for the semester
    pub gig_count: bool,
    /// Whether members are assumed to attend (we assume as much for most events)
    pub default_attend: bool,
}

#[ComplexObject]
impl Event {
    /// The gig for this event, if it is a gig
    pub async fn gig(&self, ctx: &Context<'_>) -> Result<Option<Gig>> {
        let mut conn = get_conn(ctx);
        Gig::for_event(self.id, &mut conn)
    }

    /// The attendance for the current user at this event
    pub async fn user_attendance(&self, ctx: &Context<'_>) -> Result<Attendance> {
        let mut conn = get_conn(ctx);
        let user = ctx.data_opt::<Member>();

        Attendance::for_member_at_event(user.email, self.id, &mut conn).await
    }

    /// The attendance for a specific member at this event
    pub async fn attendance(&self, ctx: &Context<'_>, member: String) -> Result<Attendance> {
        let mut conn = get_conn(ctx);
        Attendance::for_member_at_event(&member, self.id, &mut conn).await
    }

    pub async fn all_attendance(&self, ctx: &Context<'_>) -> Result<Vec<Attendance>> {
        let mut conn = get_conn(ctx);
        Attendance::for_event(self.id, &mut conn).await
    }

    pub async fn carpools(&self, ctx: &Context<'_>) -> Result<Vec<Carpool>> {
        let mut conn = get_conn(ctx);
        Carpool::for_event(self.id, &mut conn).await
    }

    pub async fn setlist(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let mut conn = get_conn(ctx);
        Song::setlist_for_event(self.id, &mut conn).await
    }
}

impl Event {
    pub async fn with_id(id: i32, mut conn: DbConn<'_>) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| format!("No event with id {}", id))
            .into()
    }

    pub async fn with_id_opt(id: i32, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM event WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn for_semester(semester: &str, mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        Semester::verify_exists(semester, conn).await?;

        sqlx::query_as!(
            Self,
            "SELECT * FROM event WHERE semester = ? ORDER BY call_time",
            semester
        )
        .fetch_all(conn)
        .await
    }

    pub fn is_gig(&self) -> bool {
        self.r#type == EventType::TUTTI_GIG || self.r#type == EventType::VOLUNTEER_GIG
    }

    pub fn ensure_no_rsvp_issue(
        &self,
        attendance: &Option<Attendance>,
        is_active: bool,
    ) -> Result<()> {
        if let Some(rsvp_issue) = self.rsvp_issue_for(attendance, is_active) {
            Err(rsvp_issue)
        } else {
            Ok(())
        }
    }

    pub fn rsvp_issue_for(
        &self,
        attendance: &Option<Attendance>,
        is_active: bool,
    ) -> Option<String> {
        if !is_active {
            Some("Member must be active to RSVP to events.".to_owned())
        } else if !attendance.map(|a| a.should_attend).unwrap_or(true) {
            None
        } else if now() + Duration::days(1) > self.call_time {
            Some("Responses are closed for this event.".to_owned())
        } else if ["Tutti Gig", "Sectional", "Rehearsal"].contains(self.r#type) {
            // TODO: update event types to constants
            Some(format!("You cannot RSVP for {} events.", self.r#type))
        } else {
            None
        }
    }

    pub async fn create(
        new_event: NewEvent,
        from_request: Option<GigRequest>,
        mut conn: DbConn<'_>,
    ) -> Result<i32> {
        if let Some(release_time) = new_event.event.release_time {
            if release_time <= new_event.event.call_time {
                return Err("Release time must be after call time".into());
            }
        }

        // TODO: redo with a `match`
        let (period, repeat_until) = if let Some(repeat) = new_event.repeat {
            let repeat_until = if repeat.period == Period::No {
                new_event.event.call_time
            } else {
                repeat
                    .repeat_until
                    .ok_or("Must supply a repeat until time if repeat is supplied")
            };

            (repeat.period, repeat.repeat_until)
        } else {
            return Err("Must supply a repeat for new events");
        };

        let call_and_release_times = Self::repeat_event_times(
            new_event.event.call_time,
            new_event.event.release_time,
            period,
            repeat_until,
        );
        if call_and_release_times.is_empty() {
            return Err("The repeat setting would render no events");
        }

        for (call_time, release_time) in call_and_release_times.iter() {
            sqlx::query!(
                "INSERT INTO event
                     (name, semester, `type`, call_time, release_time, points,
                      comments, location, gig_count, default_attend)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                new_event.event.name,
                new_event.event.semester,
                new_event.event.r#type,
                call_time,
                release_time,
                new_event.event.points,
                new_event.event.comments,
                new_event.event.location,
                new_event.event.gig_count,
                new_event.event.default_attend
            )
            .execute(conn)
            .await?;
        }

        let new_ids = sqlx::query!(
            "SELECT id FROM event ORDER BY id DESC LIMIT ?",
            call_and_release_times.len()
        )
        .fetch_all(conn)
        .await?;
        for new_id in new_ids.iter() {
            Attendance::create_for_new_event(new_id, conn).await?;
        }

        if let Some(gig) = new_event
            .gig
            .or_else(|| from_request.map(|request| request.build_new_gig()))
        {
            for new_id in new_ids {
                sqlx::query!(
                    "INSERT INTO gig
                        (event, performance_time, uniform, contact_name, contact_email,
                         contact_phone, price, `public`, summary, description)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    new_id,
                    new_event.gig.performance_time,
                    new_event.gig.uniform,
                    new_event.gig.contact_name,
                    new_event.gig.contact_email,
                    new_event.gig.contact_phone,
                    new_event.gig.price,
                    new_event.gig_public,
                    new_event.gig.summary,
                    new_event.gig.description
                )
                .fetch_all(conn)
                .await
            }
        }

        if let Some(request) = from_request {
            GigRequest::set_status(request.id, GigRequestStatus::Accepted, conn).await?;
        }

        new_ids
            .last()
            .ok_or_else(|| "Failed to find latest event ID")
    }

    pub async fn update(id: i32, update: NewEvent, mut conn: DbConn<'_>) -> Result<()> {
        let event = Self::with_id(id, conn).await?;

        sqlx::query!(
            "UPDATE event SET name = ?, semester = ?, `type` = ?, call_time = ?, release_time = ?,
                points = ?, comments = ?, location = ?, gig_count = ?, default_attend = ?
             WHERE id = ?",
            update.event.name,
            update.event.semester,
            update.event.r#type,
            update.event.call_time,
            update.event.release_time,
            update.event.points,
            update.event.comments,
            update.event.location,
            update.event.gig_count,
            update.event.default_attend,
            id
        )
        .execute(conn)
        .await?;

        if event.gig.is_some() {
            if let Some(gig) = update.gig {
                sqlx::query!(
                    "UPDATE gig SET performance_time = ?, uniform = ?, contact_name = ?, contact_email = ?,
                     contact_phone = ?, price = ?, public = ?, summary = ?, description = ?
                     WHERE event = ?", gig.performance_time, gig.uniform, gig.contact_name, gig.contact_email,
                    gig.contact_phone, gig.price, gig.public, gig.summary, gig.description, id).execute(conn).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(id: i32, mut conn: DbConn<'_>) -> Result<()> {
        // TODO: verify exists?
        Event::with_id(id, conn).await?;

        sqlx::query!("DELETE FROM event WHERE id = ?", id)
            .execute(conn)
            .await
    }

    fn repeat_event_times(
        call_time: OffsetDateTime,
        release_time: OffsetDateTime,
        period: Period,
        repeat_until: OffsetDateTime,
    ) -> Vec<(OffsetDateTime, OffsetDateTime)> {
        let increment = match period {
            Period::Daily => Duration::days(1),
            Period::Weekly => Duration::weeks(1),
            Period::Biweekly => Duration::weeks(2),
            Period::Monthly => Duration::days(30),
            Period::Yearly => Duration::days(365),
            Period::No => return vec![(call_time, release_time)],
        };

        std::iter::successors(Some((call_time, release_time)), |(c, r)| {
            Some((c + increment, r + increment)).filter(|(c, _r)| c <= repeat_until)
        })
    }
}

#[derive(InputObject)]
pub struct NewEvent {
    pub event: NewEventFields,
    pub gig: Option<NewGig>,
    pub repeat: NewEventPeriod,
}

#[derive(InputObject)]
pub struct NewEventFields {
    pub name: String,
    pub semester: String,
    pub r#type: String,
    pub call_time: GqlDateTime,
    pub release_time: Option<GqlDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: Option<bool>,
    pub default_attend: bool,
}

#[derive(InputObject)]
pub struct NewEventPeriod {
    pub period: Period,
    pub repeat_until: Option<GqlDateTime>,
}
