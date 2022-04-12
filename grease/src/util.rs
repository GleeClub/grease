use time::{OffsetDateTime, UtcOffset, Date, Month};

pub fn now() -> OffsetDateTime {
    let now = OffsetDateTime::now_utc();
    // let tz_offset = if in_dst(now) { offset!(-4) } else { offset(-5) };
    let tz_offset = UtcOffset::from_hms(-4, 0, 0).expect("Invalid offset");

    now.to_offset(tz_offset)
}

// fn in_dst(time: OffsetDateTime) -> bool {
//     // TODO: actual dates
//     // https://stackoverflow.com/questions/5590429/calculating-daylight-saving-time-from-only-date
//     let start_of_dst = Date::from_calendar_date
//     (now.month() >= Month::March && now.day() > 8)
//         && (now.month() <= November && now.day() < 7)
// }
