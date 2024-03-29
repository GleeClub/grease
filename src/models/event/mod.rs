use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use super::{DateScalar, DateTimeInput};
use crate::graphql::guards::{LoggedIn, Permission};
use crate::models::event::attendance::Attendance;
use crate::models::event::carpool::Carpool;
use crate::models::event::gig::{Gig, GigRequest, GigRequestStatus, NewGig};
use crate::models::member::Member;
use crate::models::semester::Semester;
use crate::models::song::Song;
use crate::models::DateTime;
use crate::util::current_time;

pub mod absence_request;
pub mod attendance;
pub mod carpool;
pub mod gig;
pub mod public;
pub mod uniform;

/// How often an event repeats
#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum Period {
    /// The event repeat every day
    Daily,
    /// The event repeat every week
    Weekly,
    /// The event repeat every two weeks
    Biweekly,
    /// The event repeats every thirty days
    Monthly,
    /// The event repeats every year
    Yearly,
}

/// The type of an event
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
        sqlx::query_as!(Self, "SELECT * FROM event_types ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }
}

/// An event where members are singing
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
    /// How many points attendance of this event is worth
    pub points: i64,
    /// General information or details about this event
    pub comments: String,
    /// Where this event will be held
    pub location: String,
    /// Whether this event counts toward the volunteer gig count for the semester
    pub gig_count: bool,
    /// Whether members are assumed to attend (we assume as much for most events)
    pub default_attend: bool,

    #[graphql(skip)]
    pub call_time: OffsetDateTime,
    #[graphql(skip)]
    pub release_time: Option<OffsetDateTime>,
}

#[ComplexObject]
impl Event {
    /// When members are expected to arrive to the event
    pub async fn call_time(&self) -> DateTime {
        DateTime::from(self.call_time.clone())
    }

    /// When members are probably going to be released
    pub async fn release_time(&self) -> Option<DateTime> {
        self.release_time.clone().map(DateTime::from)
    }

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
    pub async fn attendance(
        &self,
        ctx: &Context<'_>,
        member: String,
    ) -> Result<Option<Attendance>> {
        let pool: &PgPool = ctx.data_unchecked();
        Attendance::for_member_at_event_opt(&member, self.id, &pool).await
    }

    /// Attendance for all current members for the event
    #[graphql(guard = "LoggedIn")]
    pub async fn all_attendance(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            default = false,
            desc = "Whether to return an error or no attendance when not permitted"
        )]
        empty_if_not_permitted: bool,
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

    /// All carpools for this event
    pub async fn carpools(&self, ctx: &Context<'_>) -> Result<Vec<Carpool>> {
        let pool: &PgPool = ctx.data_unchecked();
        Carpool::for_event(self.id, pool).await
    }

    /// All songs we plan to sing at this event, in order
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
             FROM events WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester: &str, pool: &PgPool) -> Result<Vec<Self>> {
        // verify_exists
        Semester::with_name(semester, pool).await?;

        sqlx::query_as!(
            Self,
            "SELECT id, name, semester, \"type\", call_time as \"call_time: _\",
                  release_time as \"release_time: _\", points, comments, location,
                  gig_count, default_attend
             FROM events WHERE semester = $1 ORDER BY call_time",
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
        } else if current_time() + Duration::days(1) > self.call_time {
            Some("Responses are closed for this event".to_owned())
        } else if [
            EventType::TUTTI_GIG,
            EventType::SECTIONAL,
            EventType::REHEARSAL,
        ]
        .contains(&self.r#type.as_str())
        {
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
        let first_call_date = new_event.event.call_time.date.clone();
        if let Some(release_time) = &new_event.event.release_time {
            if release_time <= &new_event.event.call_time {
                return Err("Release time must be after call time".into());
            }
        }

        let call_and_release_times = if let Some(repeat) = new_event.repeat {
            repeat
                .event_times(
                    new_event.event.call_time.into(),
                    new_event.event.release_time.map(Into::into),
                )
                .collect()
        } else {
            vec![(
                new_event.event.call_time.into(),
                new_event.event.release_time.map(Into::into),
            )]
        };

        if call_and_release_times.is_empty() {
            return Err("The repeat setting would render no events".into());
        }

        let new_event_count = call_and_release_times.len();
        for (call_time, release_time) in call_and_release_times {
            sqlx::query!(
                "INSERT INTO events
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
            "SELECT id FROM events ORDER BY id DESC LIMIT $1",
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
            let performance_time = OffsetDateTime::from(DateTime {
                date: first_call_date,
                time: gig.performance_time,
            });

            for new_id in &new_ids {
                sqlx::query!(
                    "INSERT INTO gigs
                        (event, performance_time, uniform, contact_name, contact_email,
                         contact_phone, price, \"public\", summary, description)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    new_id,
                    performance_time,
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
            "UPDATE events SET name = $1, semester = $2, \"type\" = $3, call_time = $4, release_time = $5,
                 points = $6, comments = $7, location = $8, gig_count = $9, default_attend = $10
             WHERE id = $11",
            update.event.name,
            update.event.semester,
            update.event.r#type,
            OffsetDateTime::from(update.event.call_time.clone()),
            update.event.release_time.map(OffsetDateTime::from),
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
                let performance_time = OffsetDateTime::from(DateTime {
                    date: update.event.call_time.date,
                    time: gig.performance_time,
                });
                sqlx::query!(
                    "UPDATE gigs SET performance_time = $1, uniform = $2, contact_name = $3, contact_email = $4,
                     contact_phone = $5, price = $6, public = $7, summary = $8, description = $9
                     WHERE event = $10", performance_time, gig.uniform, gig.contact_name, gig.contact_email,
                    gig.contact_phone, gig.price, gig.public, gig.summary, gig.description, id).execute(pool).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        // verify exists
        Event::with_id(id, pool).await?;

        sqlx::query!("DELETE FROM events WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// A new event, broken into different groups of fields
#[derive(InputObject)]
pub struct NewEvent {
    /// The event fields
    pub event: NewEventFields,
    /// The gig fields, if this event is a gig
    pub gig: Option<NewGig>,
    /// How often to optionally repeat the event
    pub repeat: Option<NewEventPeriod>,
}

/// The event-specific fields on a new event
#[derive(InputObject)]
pub struct NewEventFields {
    /// The name of the event
    pub name: String,
    /// The name of the semester this event belongs to
    pub semester: String,
    /// The type of the event (see EventType)
    pub r#type: String,
    /// When members are expected to arrive to the event
    pub call_time: DateTimeInput,
    /// When members are probably going to be released
    pub release_time: Option<DateTimeInput>,
    /// How many points attendance of this event is worth
    pub points: i64,
    /// General information or details about this event
    pub comments: Option<String>,
    /// Where this event will be held
    pub location: Option<String>,
    /// Whether this event counts toward the volunteer gig count for the semester
    pub gig_count: Option<bool>,
    /// Whether members are assumed to attend (we assume as much for most events)
    pub default_attend: bool,
}

/// How often an event should repeat and until when
#[derive(InputObject)]
pub struct NewEventPeriod {
    /// How many days between repeat events
    pub period: Period,
    /// The last date the event will repeat until
    pub repeat_until: DateScalar,
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
                .filter(|(c, _r)| c.date() <= self.repeat_until.0)
        })
    }
}

#[cfg(test)]
mod tests {
    use time::{Date, Duration, Month};

    use crate::models::event::{NewEventPeriod, Period};
    use crate::models::DateScalar;

    #[test]
    fn event_times_generates_correctly() {
        let start_date = Date::from_calendar_date(2000, Month::February, 5).unwrap();
        let end_date = start_date + Duration::weeks(4);
        let period = NewEventPeriod {
            period: Period::Weekly,
            repeat_until: DateScalar(end_date),
        };
        let event_times = period
            .event_times(
                start_date.with_hms(12, 0, 0).unwrap().assume_utc(),
                Some(start_date.with_hms(13, 0, 0).unwrap().assume_utc()),
            )
            .collect::<Vec<_>>();

        assert_eq!(event_times.len(), 5);
        assert_eq!(event_times[4].0.date(), end_date);
    }
}
