pub mod models;
pub mod traits;
pub mod connection;

use mysql::prelude::ToValue;

pub use self::models::*;
pub use self::traits::*;
pub use self::connection::*;

pub fn to_value<T: ToValue>(t: T) -> String {
    t.to_value().as_sql(false)
}
