use time::{Date, Month, OffsetDateTime};

use crate::models::event::{Event, EventType};
use crate::models::semester::Semester;
use crate::models::DateScalar;

pub fn mock_semester() -> Semester {
    Semester {
        name: String::from("Spring 2000"),
        gig_requirement: 5,
        current: true,
        start_date: DateScalar(Date::from_calendar_date(2000, Month::January, 1).unwrap()),
        end_date: DateScalar(Date::from_calendar_date(2000, Month::June, 30).unwrap()),
    }
}

pub fn mock_event() -> Event {
    Event {
        id: 1,
        name: String::from("Mock Event"),
        r#type: EventType::TUTTI_GIG.to_owned(),
        semester: mock_semester().name,
        points: 35,
        comments: String::from("Let's all go sing somewhere!"),
        location: String::from("Somewhere out there"),
        gig_count: false,
        default_attend: true,
        call_time: OffsetDateTime::from_unix_timestamp(1_000_000)
            .unwrap()
            .into(),
        release_time: Some(
            OffsetDateTime::from_unix_timestamp(1_005_000)
                .unwrap()
                .into(),
        ),
    }
}
