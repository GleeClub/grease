use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use time::macros::time;
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

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct GqlDate(pub Date);

#[Scalar]
impl ScalarType for GqlDate {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::Number(epoch) = &value {
            if let Some(epoch_int) = epoch.as_i64() {
                if let Some(datetime) = current_time_from_timestamp(epoch_int) {
                    return Ok(GqlDate(datetime.date()));
                }
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        let datetime = self
            .0
            .with_time(time!(00:00))
            .assume_offset(current_offset());
        Value::Number(datetime.unix_timestamp().into())
    }
}

#[derive(sqlx::Type, Clone)]
#[sqlx(transparent)]
pub struct GqlDateTime(pub OffsetDateTime);

#[Scalar]
impl ScalarType for GqlDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::Number(epoch) = &value {
            if let Some(epoch_int) = epoch.as_i64() {
                if let Some(datetime) = current_time_from_timestamp(epoch_int) {
                    return Ok(GqlDateTime(datetime));
                }
            }
        }

        Err(InputValueError::expected_type(value))
    }

    fn to_value(&self) -> Value {
        Value::Number(self.0.unix_timestamp().into())
    }
}

fn current_offset() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
}

fn current_time_from_timestamp(timestamp: i64) -> Option<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .map(|datetime| datetime.to_offset(current_offset()))
}
