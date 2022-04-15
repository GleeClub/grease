use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    OffsetDateTime::try_now_local().expect("Failed to get system time UTC offset")
}

// fn in_dst(time: OffsetDateTime) -> bool {
//     // TODO: actual dates
//     // https://stackoverflow.com/questions/5590429/calculating-daylight-saving-time-from-only-date
//     let start_of_dst = Date::from_calendar_date
//     (now.month() >= Month::March && now.day() > 8)
//         && (now.month() <= November && now.day() < 7)
// }
