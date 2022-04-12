use time::{Date, OffsetDateTime};
use time::format_description::well_known::Rfc3339;
use time::format_description::{FormatItem, parse};
use async_graphql::{Value, InputValueError, InputValueResult, Scalar, ScalarType};
use std::lazy::SyncLazy;

pub mod document;
pub mod event;
pub mod grades;
pub mod member;
pub mod minutes;
pub mod money;
pub mod permissions;
pub mod semester;
pub mod song;
pub mod static_data;
pub mod variable;

pub struct GqlDate(pub Date);

pub static DATE_FORMAT: SyncLazy<Vec<FormatItem>> = SyncLazy::new(|| {
    parse("[year]-[month]-[day]").expect("Failed to parse date format")
});

#[Scalar]
impl ScalarType for GqlDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(date_str) = &value {
            if let Ok(date) = Date::parse(date_str, &*DATE_FORMAT) {
                return Ok(GqlDate(date));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        self.0.format(&DATE_FORMAT).map(Value::String).unwrap_or_default()
    }
}

pub struct GqlDateTime(pub OffsetDateTime);

#[Scalar]
impl ScalarType for GqlDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(date_str) = &value {
            if let Ok(date) = OffsetDateTime::parse(date_str, &Rfc3339) {
                return Ok(GqlDateTime(date));
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        self.0.format(&Rfc3339).map(Value::String).unwrap_or_default()
    }
}
