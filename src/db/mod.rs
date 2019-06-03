use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;

pub mod models;
pub mod schema;
