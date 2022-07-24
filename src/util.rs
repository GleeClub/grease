use time::{OffsetDateTime, UtcOffset};
use tz::{DateTime, TimeZone};

/// Gets the current time.
///
/// More involved than a normal implementation, because getting the
/// current timezone has proven difficult in the containerized
/// environment we deploy in.
pub fn current_time() -> OffsetDateTime {
    let timezone = TimeZone::local().expect("Failed to get current timezone");
    let now = DateTime::now(timezone.as_ref()).expect("Failed to get current time");
    let utc_offset = UtcOffset::from_whole_seconds(now.local_time_type().ut_offset())
        .expect("Failed to build timezone object");

    OffsetDateTime::from_unix_timestamp(now.unix_time())
        .expect("Failed to convert between time constructs")
        .to_offset(utc_offset)
}
