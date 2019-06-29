//! Database interaction for the API.

pub mod connection;
pub mod models;
pub mod traits;

use mysql::prelude::ToValue;

pub use self::connection::*;
pub use self::models::*;
pub use self::traits::*;

/// Convert model fields to strings in SQL-compliant formats.
///
/// A handy helper method for building SQL queries.
pub fn to_value<T: ToValue>(t: T) -> String {
    t.to_value().as_sql(false)
}
