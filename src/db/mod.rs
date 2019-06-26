pub mod connection;
pub mod models;
pub mod traits;

use mysql::prelude::ToValue;

pub use self::connection::*;
pub use self::models::*;
pub use self::traits::*;

pub fn to_value<T: ToValue>(t: T) -> String {
    t.to_value().as_sql(false)
}
