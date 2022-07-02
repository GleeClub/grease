use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use time::format_description::FormatItem;
use time::macros::format_description;
use time::{Date, OffsetDateTime, UtcOffset};

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

pub const DATE_FORMAT: &[FormatItem] = format_description!("%Y-%m-%d");
pub const DATETIME_FORMAT: &[FormatItem] = format_description!("%Y-%m-%dT%H:%M:%SZ");

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct GqlDate(pub Date);

#[Scalar]
impl ScalarType for GqlDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(date_str) = &value {
            if let Ok(date) = Date::parse(date_str, DATE_FORMAT) {
                return Ok(GqlDate(date));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.format(DATE_FORMAT).unwrap())
    }
}

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct GqlDateTime(pub OffsetDateTime);

#[Scalar]
impl ScalarType for GqlDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(date_str) = &value {
            if let Ok(date) = OffsetDateTime::parse(date_str, DATETIME_FORMAT) {
                return Ok(GqlDateTime(date));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::String(
            self.0
                .to_offset(UtcOffset::UTC)
                .format(DATETIME_FORMAT)
                .unwrap(),
        )
    }
}
