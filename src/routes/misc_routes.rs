//! All other miscellaneous routes.

use auth::User;
use db::models::*;
use error::GreaseResult;
use serde_json::{json, Value};

/// Get a variable.
///
/// ## Path Parameters:
///   * key: string (*required*) - The name of the variable
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a [Variable](crate::db::models::Variable) or null.
pub fn get_variable(key: String, mut user: User) -> GreaseResult<Value> {
    Variable::load(&key, &mut user.conn).map(|var| json!(var))
}

/// Set a variable.
///
/// ## Path Parameters:
///   * key: string (*required*) - The name of the variable
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Input Format:
///
/// Expects a [NewValue](crate::db::models::NewValue).
///
/// ## Return Format:
///
/// ```json
/// {
///     "oldValue": string?
/// }
/// ```
///
/// Returns an object with the old value if the variable was already set,
/// or null if it wasn't.
pub fn set_variable(key: String, (new_value, mut user): (NewValue, User)) -> GreaseResult<Value> {
    Variable::set(key, new_value.value, &mut user.conn)
        .map(|old_val| json!({
            "oldValue": old_val
        }))
}

/// Unset a variable.
///
/// ## Path Parameters:
///   * key: string (*required*) - The name of the variable
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// ```json
/// {
///     "oldValue": string?
/// }
/// ```
///
/// Returns an object with the old value if the variable was set,
/// or null if it wasn't.
pub fn unset_variable(key: String, mut user: User) -> GreaseResult<Value> {
    Variable::unset(&key, &mut user.conn)
        .map(|old_val| json!({
            "oldValue": old_val
        }))
}
