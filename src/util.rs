use time::OffsetDateTime;

pub fn current_time() -> OffsetDateTime {
    OffsetDateTime::now_local().expect("Failed to get current timezone")
}
