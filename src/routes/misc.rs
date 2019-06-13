use app_route::AppRoute;
use auth::User;
use error::{GreaseError, GreaseResult};
use db::models::*;
use serde_json::{json, Value};
use super::{OptionalIdQuery, OptionalEmailQuery};
use crate::check_for_permission;

#[derive(AppRoute, Debug)]
#[route("/variables/:key")]
pub struct GetVariableRequest {
    pub key: String,
}

pub fn get_variable(req: GetVariableRequest, user: User) -> GreaseResult<Value> {
    Variable::load(&req.key, &user.conn).map(|var| json!(var))
}

#[derive(AppRoute, Debug)]
#[route("/variables/:key/:value")]
pub struct SetVariableRequest {
    pub key: String,
    pub value: String,
}

pub fn set_variable(req: SetVariableRequest, user: User) -> GreaseResult<Value> {
    Variable::set(req.key, req.value, &user.conn).map(|old_val| json!(old_val))
}
