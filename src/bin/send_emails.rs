use anyhow::Result;
use grease::db::DbConn;
use grease::email::event::email_for_event;
use grease::models::event::Event;
use grease::models::semester::Semester;
use grease::util::{gql_err_to_anyhow, now};
use time::{Duration, OffsetDateTime};

#[tokio::main]
pub async fn main() -> Result<()> {
    let conn = DbConn::connect().await?;
    let current_semester = Semester::get_current(&conn)
        .await
        .map_err(gql_err_to_anyhow)?;
    let all_events = Event::for_semester(&current_semester.name, &conn)
        .await
        .map_err(gql_err_to_anyhow)?;
    let since = now() - Duration::hours(1);

    for event in filter_unaddressed_events(all_events, since) {
        email_for_event(&event, &conn)
            .await
            .map_err(gql_err_to_anyhow)?
            .send()
            .await?;
    }

    Ok(())
}

fn filter_unaddressed_events(
    events: Vec<Event>,
    since: OffsetDateTime,
) -> impl Iterator<Item = Event> {
    let two_days_from_now = now() + Duration::days(2);
    let two_days_from_last_checked = since + Duration::days(2);

    events.into_iter().filter(move |event| {
        event.call_time.0 < two_days_from_now && event.call_time.0 > two_days_from_last_checked
    })
}
