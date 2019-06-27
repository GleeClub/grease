use auth::User;
use db::models::*;
use error::GreaseResult;
use serde_json::{json, Value};
use serde::Deserialize;

pub fn get_variable(key: String, mut user: User) -> GreaseResult<Value> {
    Variable::load(&key, &mut user.conn).map(|var| json!(var))
}

#[derive(Deserialize, grease_derive::Extract)]
pub struct NewValue {
    pub value: String,
}

pub fn set_variable(key: String, (new_value, mut user): (NewValue, User)) -> GreaseResult<Value> {
    Variable::set(key, new_value.value, &mut user.conn).map(|old_val| json!(old_val))
}

pub fn unset_variable(key: String, mut user: User) -> GreaseResult<Value> {
    Variable::unset(&key, &mut user.conn).map(|old_val| json!(old_val))
}
