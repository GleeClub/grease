//! All other miscellaneous routes.

use auth::User;
use db::*;
use diesel::prelude::*;
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
pub fn set_variable(key: String, new_value: NewValue, mut user: User) -> GreaseResult<Value> {
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
pub fn static_data() -> GreaseResult<Value> {
    let mut conn = connect_to_db()?;
    use db::schema::*;

    Ok(json!({
        "mediaTypes": MediaType::load_all(&mut conn)?,
        "uniforms": Uniform::load_all(&mut conn)?,
        "permissions": permission::table.order_by(permission::name.asc()).load::<Permission>(&mut conn)?,
        "roles": role::table.order_by(role::rank.asc()).load::<Role>(&mut conn)?,
        "eventTypes": event_type::table.order_by(event_type::name.asc()).load::<EventType>(&mut conn)?,
        "sections": section_type::table
            .order_by(section_type::name.asc())
            .load::<SectionType>(&mut conn)?
            .into_iter()
            .map(|type_| type_.name)
            .collect::<Vec<_>>(),
        "transactionTypes": transaction_type::table
            .order_by(transaction_type::name.asc())
            .load::<TransactionType>(&mut conn)?
            .into_iter()
            .map(|type_| type_.name)
            .collect::<Vec<_>>(),
    }))
}
