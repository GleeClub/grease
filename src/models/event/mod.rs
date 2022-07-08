use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use crate::graphql::guards::{LoggedIn, Permission};
use crate::models::event::attendance::Attendance;
use crate::models::event::carpool::Carpool;
use crate::models::event::gig::{Gig, GigRequest, GigRequestStatus, NewGig};
use crate::models::member::Member;
use crate::models::semester::Semester;
use crate::models::song::Song;
use crate::models::GqlDateTime;
use crate::util::current_time;

pub mod absence_request;
pub mod attendance;
pub mod carpool;
pub mod gig;
pub mod public;
pub mod uniform;

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Period {
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
    pub weight: i64,
}

impl EventType {
    pub const REHEARSAL: &'static str = "Rehearsal";
    pub const SECTIONAL: &'static str = "Sectional";
    pub const VOLUNTEER_GIG: &'static str = "Volunteer Gig";
    pub const TUTTI_GIG: &'static str = "Tutti Gig";
    pub const OMBUDS: &'static str = "Ombuds";
    pub const OTHER: &'static str = "Other";

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM event_type ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Event {
    /// The ID of the event
    pub id: i64,
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
    pub points: i64,
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
        let pool: &PgPool = ctx.data_unchecked();
        Gig::for_event(self.id, pool).await
    }

    /// The attendance for the current user at this event
    pub async fn user_attendance(&self, ctx: &Context<'_>) -> Result<Option<Attendance>> {
        let pool: &PgPool = ctx.data_unchecked();

        if let Some(user) = ctx.data_opt::<Member>() {
            Attendance::for_member_at_event_opt(&user.email, self.id, pool).await
        } else {
            Ok(None)
        }
    }

    /// The attendance for a specific member at this event
    pub async fn attendance(&self, ctx: &Context<'_>, member: String) -> Result<Attendance> {
        let pool: &PgPool = ctx.data_unchecked();
        Attendance::for_member_at_event(&member, self.id, &pool).await
    }

    // TODO: permissions (should return empty list if not allowed, for convenience?)
    #[graphql(guard = "LoggedIn")]
    pub async fn all_attendance(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = false)] empty_if_not_permitted: bool,
    ) -> Result<Vec<Attendance>> {
        let pool: &PgPool = ctx.data_unchecked();
        let user: &Member = ctx.data_unchecked();
        if !Permission::EDIT_ATTENDANCE
            .for_type(&self.r#type)
            .granted_to(&user.email, pool)
            .await?
        {
            if empty_if_not_permitted {
                return Ok(vec![]);
            } else {
                return Err(Permission::EDIT_ATTENDANCE.error());
            }
        }

        Attendance::for_event(self.id, pool).await
    }

    pub async fn carpools(&self, ctx: &Context<'_>) -> Result<Vec<Carpool>> {
        let pool: &PgPool = ctx.data_unchecked();
        Carpool::for_event(self.id, pool).await
    }

    pub async fn setlist(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let pool: &PgPool = ctx.data_unchecked();
        Song::setlist_for_event(self.id, pool).await
    }
}

impl Event {
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No event with id {}", id))
            .map_err(Into::into)
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, semester, type, call_time as \"call_time: _\",
                  release_time as \"release_time: _\", points, comments, location,
                  gig_count, default_attend
             FROM event WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester: &str, pool: &PgPool) -> Result<Vec<Self>> {
        // TODO: verify_exists
        Semester::with_name(semester, pool).await?;

        sqlx::query_as!(
            Self,
            "SELECT id, name, semester, \"type\", call_time as \"call_time: _\",
                  release_time as \"release_time: _\", points, comments, location,
                  gig_count, default_attend
             FROM event WHERE semester = $1 ORDER BY call_time",
            semester
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub fn is_gig(&self) -> bool {
        &self.r#type == EventType::TUTTI_GIG || &self.r#type == EventType::VOLUNTEER_GIG
    }

    pub fn ensure_no_rsvp_issue(
        &self,
        attendance: Option<&Attendance>,
        is_active: bool,
    ) -> Result<()> {
        if let Some(rsvp_issue) = self.rsvp_issue_for(attendance, is_active) {
            Err(rsvp_issue.into())
        } else {
            Ok(())
        }
    }

    pub fn rsvp_issue_for(
        &self,
        attendance: Option<&Attendance>,
        is_active: bool,
    ) -> Option<String> {
        if !is_active {
            Some("Member must be active to RSVP to events".to_owned())
        } else if !attendance.map(|a| a.should_attend).unwrap_or(true) {
            None
        } else if current_time() + Duration::days(1) > self.call_time.0 {
            Some("Responses are closed for this event".to_owned())
        } else if ["Tutti Gig", "Sectional", "Rehearsal"].contains(&self.r#type.as_str()) {
            // TODO: update event types to constants
            Some(format!("You cannot RSVP for {} events", self.r#type))
        } else {
            None
        }
    }

    pub async fn create(
        new_event: NewEvent,
        from_request: Option<GigRequest>,
        pool: &PgPool,
    ) -> Result<i64> {
        if let Some(release_time) = &new_event.event.release_time {
            if &release_time.0 <= &new_event.event.call_time.0 {
                return Err("Release time must be after call time".into());
            }
        }

        let call_and_release_times = if let Some(repeat) = new_event.repeat {
            repeat
                .event_times(
                    new_event.event.call_time.0,
                    new_event.event.release_time.map(|rt| rt.0),
                )
                .collect()
        } else {
            vec![(
                new_event.event.call_time.0,
                new_event.event.release_time.map(|rt| rt.0),
            )]
        };

        if call_and_release_times.is_empty() {
            return Err("The repeat setting would render no events".into());
        }

        let new_event_count = call_and_release_times.len();
        for (call_time, release_time) in call_and_release_times {
            sqlx::query!(
                "INSERT INTO event
                     (name, semester, \"type\", call_time, release_time, points,
                      comments, location, gig_count, default_attend)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
            .execute(pool)
            .await?;
        }

        let new_ids = sqlx::query_scalar!(
            "SELECT id FROM event ORDER BY id DESC LIMIT $1",
            new_event_count as i64
        )
        .fetch_all(pool)
        .await?;
        for new_id in &new_ids {
            Attendance::create_for_new_event(*new_id, pool).await?;
        }

        let gig = if new_event.gig.is_some() {
            new_event.gig
        } else if let Some(request) = &from_request {
            Some(request.build_new_gig(pool).await?)
        } else {
            None
        };

        if let Some(gig) = gig {
            for new_id in &new_ids {
                sqlx::query!(
                    "INSERT INTO gig
                        (event, performance_time, uniform, contact_name, contact_email,
                         contact_phone, price, \"public\", summary, description)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    new_id,
                    gig.performance_time.0,
                    gig.uniform,
                    gig.contact_name,
                    gig.contact_email,
                    gig.contact_phone,
                    gig.price,
                    gig.public,
                    gig.summary,
                    gig.description
                )
                .fetch_all(pool)
                .await?;
            }
        }

        if let Some(request) = from_request {
            GigRequest::set_status(request.id, GigRequestStatus::Accepted, pool).await?;
        }

        new_ids
            .into_iter()
            .last()
            .ok_or_else(|| "Failed to find latest event ID".into())
    }

    pub async fn update(id: i64, update: NewEvent, pool: &PgPool) -> Result<()> {
        Self::with_id(id, pool).await?;

        sqlx::query!(
            "UPDATE event SET name = $1, semester = $2, \"type\" = $3, call_time = $4, release_time = $5,
                 points = $6, comments = $7, location = $8, gig_count = $9, default_attend = $10
             WHERE id = $11",
            update.event.name,
            update.event.semester,
            update.event.r#type,
            update.event.call_time.0,
            update.event.release_time.map(|rt| rt.0),
            update.event.points,
            update.event.comments,
            update.event.location,
            update.event.gig_count,
            update.event.default_attend,
            id
        )
        .execute(pool)
        .await?;

        if Gig::for_event(id, pool).await?.is_some() {
            if let Some(gig) = update.gig {
                sqlx::query!(
                    "UPDATE gig SET performance_time = $1, uniform = $2, contact_name = $3, contact_email = $4,
                     contact_phone = $5, price = $6, public = $7, summary = $8, description = $9
                     WHERE event = $10", gig.performance_time.0, gig.uniform, gig.contact_name, gig.contact_email,
                    gig.contact_phone, gig.price, gig.public, gig.summary, gig.description, id).execute(pool).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        // TODO: verify exists?
        Event::with_id(id, pool).await?;

        sqlx::query!("DELETE FROM event WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewEvent {
    pub event: NewEventFields,
    pub gig: Option<NewGig>,
    pub repeat: Option<NewEventPeriod>,
}

#[derive(InputObject)]
pub struct NewEventFields {
    pub name: String,
    pub semester: String,
    pub r#type: String,
    pub call_time: GqlDateTime,
    pub release_time: Option<GqlDateTime>,
    pub points: i64,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: Option<bool>,
    pub default_attend: bool,
}

#[derive(InputObject)]
pub struct NewEventPeriod {
    pub period: Period,
    pub repeat_until: GqlDateTime,
}

impl NewEventPeriod {
    pub fn event_times(
        self,
        call_time: OffsetDateTime,
        release_time: Option<OffsetDateTime>,
    ) -> impl Iterator<Item = (OffsetDateTime, Option<OffsetDateTime>)> {
        let increment = match self.period {
            Period::Daily => Duration::days(1),
            Period::Weekly => Duration::weeks(1),
            Period::Biweekly => Duration::weeks(2),
            Period::Monthly => Duration::days(30),
            Period::Yearly => Duration::days(365),
        };

        std::iter::successors(Some((call_time, release_time)), move |(c, r)| {
            Some((*c + increment, r.map(|r| r + increment)))
                .filter(|(c, _r)| *c <= self.repeat_until.0)
        })
    }
}
