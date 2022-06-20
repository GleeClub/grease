use grease::email::event::email_for_event;
use grease::error::GreaseResult;
use grease::models::event::Event;
use grease::models::semester::Semester;
use grease::util::{connect_to_db, now};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use time::{Duration, OffsetDateTime};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = service_fn(handle);
    lambda_runtime::run(handler).await
}

#[derive(Deserialize, Serialize)]
struct EmptyValue {}

async fn handle(_request: LambdaEvent<EmptyValue>) -> GreaseResult<EmptyValue> {
    // TODO: handle commit
    let pool = connect_to_db().await?;
    let since = now()? - Duration::hours(1);

    for _event in events_to_notify_about(&pool, since).await {
        // let email = email_for_event(&event, &pool).await?;
        // email.send().await?;
    }

    Ok(EmptyValue {})
}

async fn events_to_notify_about(
    pool: &MySqlPool,
    since: OffsetDateTime,
) -> async_graphql::Result<impl Iterator<Item = Event>> {
    let current_semester = Semester::get_current(&pool).await?;
    let all_events = Event::for_semester(&current_semester.name, &pool).await?;

    let two_days_from_now = now()? + Duration::days(2);
    let two_days_from_last_checked = since + Duration::days(2);

    Ok(all_events.into_iter().filter(move |event| {
        event.call_time.0 < two_days_from_now && event.call_time.0 > two_days_from_last_checked
    }))
}
