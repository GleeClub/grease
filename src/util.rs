use time::{OffsetDateTime, UtcOffset};

/// Gets the current time.
///
/// More involved than a normal implementation, because getting the
/// current timezone has proven difficult in the containerized
/// environment we deploy in.
pub fn current_time() -> OffsetDateTime {
    OffsetDateTime::now_utc().to_offset(local_offset())
}

pub fn local_offset() -> UtcOffset {
    UtcOffset::current_local_offset().expect("Failed to get current local timezone offset")
}
