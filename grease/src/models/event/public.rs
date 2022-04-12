use async_graphql::{ComplexObject, Result, SimpleObject};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::DbConn;

#[derive(SimpleObject)]
pub struct PublicEvent {
    pub id: i32,
    pub name: String,
    pub start_time: OffsetDateTime,
    pub end_time: Option<OffsetDateTime>,
    // needs default
    pub location: String,
    // needs default
    pub summary: String,
    // needs default
    pub description: String,
}

#[ComplexObject]
impl PublicEvent {
    const DATETIME_FORMAT: &'static str = "%Y%m%dT%H%M%SZ";

    pub async fn invite(&self) -> Result<String> {
        let now = OffsetDateTime::now_local()
            .map_err(|err| format!("Failed to get current time in local time zone: {}", err));
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
            self.end_time,
            self.start_time,
            self.location,
            self.description,
            Uuid::new_v4(),
        );

        Ok(format!(
            "data:text/calendar;base64,{}",
            base64::encode(&details)
        ))
    }
}

impl PublicEvent {
    pub async fn all_for_current_semester(conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event.id, event.name, gig.performance_time as start_time,
                event.release_time as end_time, event.location, gig.summary, gig.description
             FROM event
             INNER JOIN gig ON event.id = gig.event
             WHERE gig.public = true"
        )
        .fetch_all(conn)
        .await
    }
}
