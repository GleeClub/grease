use async_graphql::{SimpleObject, ComplexObject};
use chrono::NaiveDateTime;
use crate::db_conn::DbConn;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct PublicEvent {
    pub id: isize,
    pub name: String,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
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

    pub async fn invite(&self) -> String {
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
            chrono::Local::now().naive_local(),
            self.summary,
            self.end_time,
            self.start_time,
            self.location,
            self.description,
            Uuid::new_v4(),
        );

        format!("data:text/calendar;base64,{}", base64::encode(&details))
    }

}

impl PublicEvent {
    pub async fn all_for_current_semester(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT e.id, e.name, g.performance_time as start_time,
                e.release_time as end_time, e.location, g.summary,
                g.description
             FROM event as e
             INNER JOIN gig as g ON e.id = g.event
             WHERE g.public = true").query_all(conn).await
    }

}


