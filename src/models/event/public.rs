use async_graphql::{ComplexObject, Result, SimpleObject};
use sqlx::PgPool;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
use uuid::Uuid;

use crate::models::GqlDateTime;
use crate::util::current_time;

pub const DATETIME_FORMAT: &[FormatItem] =
    format_description!("[year][month][day]T[hour][minute][second]Z");

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct PublicEvent {
    pub id: i64,
    pub name: String,
    pub start_time: GqlDateTime,
    pub end_time: Option<GqlDateTime>,
    pub location: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[ComplexObject]
impl PublicEvent {
    pub async fn invite(&self) -> String {
        let now = Self::format_datetime(&current_time());
        let start_time = Self::format_datetime(&self.start_time.0);
        let end_time = self
            .end_time
            .as_ref()
            .map(|et| Self::format_datetime(&et.0))
            .unwrap_or_default();
        let location = self.location.clone().unwrap_or_default();
        let summary = self.summary.clone().unwrap_or_default();
        let description = self.description.clone().unwrap_or_default();

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
            summary,
            end_time,
            start_time,
            location,
            description,
            Uuid::new_v4(),
        );

        format!("data:text/calendar;base64,{}", base64::encode(&details))
    }
}

impl PublicEvent {
    pub async fn all_for_current_semester(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event.id, event.name, gig.performance_time as \"start_time: _\",
                 event.release_time as \"end_time: _\", event.location, gig.summary, gig.description
             FROM event
             INNER JOIN gig ON event.id = gig.event
             WHERE gig.public = true AND event.semester =
                 (SELECT name FROM semester WHERE current = true)"
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
