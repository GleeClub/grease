use std::cmp::Ordering;

use async_graphql::{
    InputObject, InputValueError, InputValueResult, Scalar, ScalarType, SimpleObject, Value,
};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{Date, OffsetDateTime, Time};

use crate::util::local_offset;

pub mod event;
pub mod grades;
pub mod link;
pub mod member;
pub mod minutes;
pub mod money;
pub mod permissions;
pub mod semester;
pub mod song;
pub mod static_data;
pub mod variable;

static DATE_FORMAT: &'static [FormatItem<'static>] = format_description!("[year]-[month]-[day]");
static TIME_FORMAT: &'static [FormatItem<'static>] = format_description!("[hour]:[minute]");

#[derive(sqlx::Type, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[sqlx(transparent)]
pub struct DateScalar(pub Date);

#[Scalar]
impl ScalarType for DateScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(date) = &value {
            if let Ok(date) = Date::parse(&date, DATE_FORMAT) {
                return Ok(DateScalar(date));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.format(DATE_FORMAT).unwrap())
    }
}

#[derive(sqlx::Type, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(transparent)]
pub struct TimeScalar(pub Time);

#[Scalar]
impl ScalarType for TimeScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(time) = &value {
            if let Ok(time) = Time::parse(&time, TIME_FORMAT) {
                return Ok(TimeScalar(time));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.format(TIME_FORMAT).unwrap())
    }
}

#[derive(SimpleObject, Clone, PartialEq, Eq, PartialOrd)]
pub struct DateTime {
    /// The date part of the datetime
    pub date: DateScalar,
    /// The time part of the datetime
    pub time: TimeScalar,
}

#[derive(InputObject, Clone, PartialEq, Eq, PartialOrd)]
pub struct DateTimeInput {
    /// The date part of the datetime
    pub date: DateScalar,
    /// The time part of the datetime
    pub time: TimeScalar,
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date
            .0
            .cmp(&other.date.0)
            .then_with(|| self.time.0.cmp(&other.time.0))
    }
}

impl From<DateTimeInput> for DateTime {
    fn from(datetime: DateTimeInput) -> Self {
        DateTime {
            date: datetime.date,
            time: datetime.time,
        }
    }
}

impl From<OffsetDateTime> for DateTime {
    fn from(datetime: OffsetDateTime) -> Self {
        let datetime = datetime.to_offset(local_offset());

        DateTime {
            date: DateScalar(datetime.date()),
            time: TimeScalar(datetime.time()),
        }
    }
}

impl From<DateTime> for OffsetDateTime {
    fn from(datetime: DateTime) -> Self {
        datetime
            .date
            .0
            .with_time(datetime.time.0)
            .assume_offset(local_offset())
    }
}

impl From<DateTimeInput> for OffsetDateTime {
    fn from(datetime: DateTimeInput) -> Self {
        OffsetDateTime::from(DateTime {
            date: datetime.date,
            time: datetime.time,
        })
    }
}

impl From<OffsetDateTime> for DateTimeInput {
    fn from(datetime: OffsetDateTime) -> Self {
        let datetime = DateTime::from(datetime);
        DateTimeInput {
            date: datetime.date,
            time: datetime.time,
        }
    }
}
