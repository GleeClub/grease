use async_graphql::{ComplexObject, Result, SimpleObject};
use sqlx::PgPool;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

use crate::models::DateTime;
use crate::util::current_time;

const DATETIME_FORMAT: &[FormatItem] =
    format_description!("[year][month][day]T[hour][minute][second]Z");

/// Events that are visible to the public
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PublicEvent {
    /// The ID of the event
    pub id: i64,
    /// The name of the event
    pub name: String,
    /// The location of the event
    pub location: String,
    // TODO: name _and_ summary?
    /// A short summary of the event
    pub summary: String,
    /// A short description of the event
    pub description: String,

    #[graphql(skip)]
    pub start_time: OffsetDateTime,
    #[graphql(skip)]
    pub end_time: Option<OffsetDateTime>,
}

#[ComplexObject]
impl PublicEvent {
    /// When this event will start
    pub async fn start_time(&self) -> DateTime {
        self.start_time.clone().into()
    }

    /// When this event will end
    pub async fn end_time(&self) -> Option<DateTime> {
        self.end_time.clone().map(Into::into)
    }

    /// An invite to add this event to your calendar
    pub async fn invite(&self) -> String {
        let now = Self::format_datetime(&current_time());
        let start_time = Self::format_datetime(&self.start_time.clone().into());
        let end_time = self
            .end_time
            .as_ref()
            .map(|et| Self::format_datetime(&et.clone().into()))
            .unwrap_or_default();

        let details = format!(
            "VERSION:2.0\n\
             PRODID:ICALENDAR-RS\n\
             CALSCALE:GREGORIAN\n\
             BEGIN:VEVENT\n\
             DTSTAMP:{}\n\
             DESCRIPTION:{}\n\
             DTEND:{}\n\
             DTSTART:{}\n\
             LOCATION:{}\n\
             SUMMARY:{}\n\
             UID:{}\n\
             END:VEVENT\n\
             END:VCALENDAR\n\
            ",
            now,
            self.summary,
            end_time,
            start_time,
            self.location,
            self.description,
            Uuid::new_v4(),
        );

        format!("data:text/calendar;base64,{}", base64::encode(&details))
    }
}

impl PublicEvent {
    pub async fn all_for_current_semester(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT events.id, events.name, gigs.performance_time as \"start_time: _\",
                 events.release_time as \"end_time: _\", events.location, gigs.summary, gigs.description
             FROM events
             INNER JOIN gigs ON events.id = gigs.event
             WHERE gigs.public = true AND events.semester =
                 (SELECT name FROM semesters WHERE current = true)"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub fn format_datetime(datetime: &OffsetDateTime) -> String {
        datetime
            .to_offset(UtcOffset::UTC)
            .format(DATETIME_FORMAT)
            .unwrap()
    }
}
