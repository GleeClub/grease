//! All other miscellaneous routes.

use auth::User;
use db::*;
use error::GreaseResult;
use pinto::query_builder::Order;
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
        .map(|old_val| json!({ "oldValue": old_val }))
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
    Variable::unset(&key, &mut user.conn).map(|old_val| json!({ "oldValue": old_val }))
}

/// Loads all of the static data for the club.
///
/// ## Return Format:
///
/// ```json
/// {
///     "eventTypes": [EventType],
///     "mediaTypes": [MediaType],
///     "permissions": [Permission],
///     "roles": [Role],
///     "sections": [string],
///     "transactionTypes": [string],
///     "uniforms": [Uniform]
/// }
/// ```
///
/// Returns an object with all of the static data of the website. See
/// [EventType](crate::db::models::EventType),
/// [MediaType](crate::db::models::MediaType),
/// [Permission](crate::db::models::Permission),
/// [Role](crate::db::models::Role), and
/// [Uniform](crate::db::models::Uniform)
/// for the formats of each field.
pub fn static_data(mut conn: DbConn) -> GreaseResult<Value> {
    Ok(json!({
        "mediaTypes": MediaType::load_all(&mut conn)?,
        "uniforms": Uniform::load_all(&mut conn)?,
        "permissions": conn
            .load::<Permission>(&Permission::select_all_in_order("name", Order::Asc))?,
        "roles": conn
            .load::<Role>(&Role::select_all_in_order("rank", Order::Asc))?,
        "eventTypes": conn
            .load::<EventType>(&EventType::select_all_in_order("name", Order::Asc))?,
        "sections": conn
            .load::<SectionType>(&SectionType::select_all_in_order("name", Order::Asc))?
            .into_iter()
            .map(|section_type| section_type.name)
            .collect::<Vec<_>>(),
        "transactionTypes": conn
            .load::<TransactionType>(&TransactionType::select_all_in_order("name", Order::Asc))?
            .into_iter()
            .map(|transaction_type| transaction_type.name)
            .collect::<Vec<_>>()
    }))
}
