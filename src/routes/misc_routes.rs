use auth::User;
use db::models::*;
use error::GreaseResult;
use serde_json::{json, Value};

pub fn get_variable(key: String, mut user: User) -> GreaseResult<Value> {
    Variable::load(&key, &mut user.conn).map(|var| json!(var))
}

pub fn set_variable(key: String, value: String, mut user: User) -> GreaseResult<Value> {
    Variable::set(key, value, &mut user.conn).map(|old_val| json!(old_val))
}