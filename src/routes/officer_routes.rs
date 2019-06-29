//! All officer-focused routes.

use super::basic_success;
use crate::check_for_permission;
use auth::*;
use db::*;
use db::models::member::MemberForSemester;
use error::*;
use pinto::query_builder::*;
use serde_json::{json, Value};

/// Get a single announcement.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the announcement
///
/// ## Return Format:
///
/// Returns an [Announcement](crate::db::models::Announcement).
pub fn get_announcement(id: i32, mut user: User) -> GreaseResult<Value> {
    Announcement::load(id, &mut user.conn).map(|announcement| json!(announcement))
}

/// Get all announcements that aren't archived for the current semester.
///
/// ## Query Parameters:
///   * all: boolean (*optional*) - Simply return all announcements ever made
///
/// ## Return Format:
///
/// Returns a list of [Announcement](crate::db::models::Announcement)s.
pub fn get_announcements(all: Option<bool>, mut user: User) -> GreaseResult<Value> {
    if all.unwrap_or(false) {
        Announcement::load_all(&mut user.conn).map(|announcements| json!(announcements))
    } else {
        let current_semester = Semester::load_current(&mut user.conn)?;
        Announcement::load_all_for_semester(&current_semester.name, &mut user.conn)
            .map(|announcements| json!(announcements))
    }
}

/// Make a new announcement.
///
/// ## Input Format:
///
/// Expects a [NewAnnouncement](crate::db::models::NewAnnouncement).
pub fn make_new_announcement(
    (mut user, new_announcement): (User, NewAnnouncement),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-announcements");
    let current_semester = Semester::load_current(&mut user.conn)?;
    Announcement::insert(
        &new_announcement.content,
        &user.member.member.email,
        &current_semester.name,
        &mut user.conn,
    )
    .map(|new_id| json!({ "id": new_id }))
}

/// Archive an announcement.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the announcement
pub fn archive_announcement(announcement_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-announcements");
    Announcement::archive(announcement_id, &mut user.conn).map(|_| basic_success())
}

/// Get a single Google Doc.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the Google Doc
///
/// ## Return Format:
///
/// Returns an [GoogleDoc](crate::db::models::GoogleDoc).
pub fn get_google_doc(name: String, mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load(&name, &mut user.conn).map(|doc| json!(doc))
}

/// Get all of the Google Docs.
///
/// ## Return Format:
///
/// Returns a list of [GoogleDoc](crate::db::models::GoogleDoc)s.
pub fn get_google_docs(mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load_all(&mut user.conn).map(|docs| json!(docs))
}

/// Create a new Google Doc.
///
/// ## Input Format:
///
/// Expects a [GoogleDoc](crate::db::models::GoogleDoc).
pub fn new_google_doc((mut user, new_doc): (User, GoogleDoc)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::insert(&new_doc, &mut user.conn).map(|id| json!({ "id": id }))
}

/// Update an existing Google Doc.
///
/// ## Path Parameters:
///   * name: string (*required*) - The current name of the Google Doc
///
/// ## Input Format:
///
/// Expects a [GoogleDoc](crate::db::models::GoogleDoc).
pub fn modify_google_doc(
    name: String,
    (mut user, changed_doc): (User, GoogleDoc),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::update(&name, &changed_doc, &mut user.conn).map(|_| basic_success())
}

/// Delete a Google Doc.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the Google Doc
pub fn delete_google_doc(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::delete(&name, &mut user.conn).map(|_| basic_success())
}

/// Get a single meeting's minutes.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
///
/// ## Return Format:
///
/// Returns a [MeetingMinutes](crate::db::models::MeetingMinutes).
pub fn get_meeting_minutes(minutes_id: i32, mut user: User) -> GreaseResult<Value> {
    let can_view_complete_minutes = user.has_permission("view-complete-minutes", None);
    MeetingMinutes::load(minutes_id, &mut user.conn)
        .map(|minutes| minutes.to_json(can_view_complete_minutes))
}

/// Returns all meeting minutes ever recorded.
///
/// ## Return Format:
///
/// Returns a list of [MeetingMinutes](crate::db::models::MeetingMinutes) objects.
pub fn get_all_meeting_minutes(mut user: User) -> GreaseResult<Value> {
    let can_view_complete_minutes = user.has_permission("view-complete-minutes", None);
    MeetingMinutes::load_all(&mut user.conn).map(|all_minutes| {
        all_minutes
            .into_iter()
            .map(|minutes| minutes.to_json(can_view_complete_minutes))
            .collect::<Vec<_>>()
            .into()
    })
}

/// Modify an existing meeting's minutes.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
///
/// ## Input Format:
///
/// Expects an [UpdatedMeetingMinutes](crate::db::models::UpdatedMeetingMinutes).
pub fn modify_meeting_minutes(
    minutes_id: i32,
    (mut user, changed_minutes): (User, UpdatedMeetingMinutes),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::update(minutes_id, &changed_minutes, &mut user.conn).map(|_| basic_success())
}

/// Create a new meeting minutes.
///
/// ## Input Format:
///
/// Expects a [NewMeetingMinutes](crate::db::models::NewMeetingMinutes).
pub fn new_meeting_minutes(
    (mut user, new_minutes): (User, NewMeetingMinutes),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::create(&new_minutes, &mut user.conn).map(|id| json!({ "id": id }))
}

/// Get all of a members todo actions.
///
/// ## Return Format:
///
/// Returns a list of [Todo](crate::db::models::Todo)s.
pub fn get_todos(mut user: User) -> GreaseResult<Value> {
    Todo::load_all_for_member(&user.member.member.email, &mut user.conn).map(|todos| json!(todos))
}

/// Add a todo action for multiple members to have to complete.
///
/// ## Input Format:
///
/// Expects a [NewTodo](crate::db::models::NewTodo).
pub fn add_todo_for_members((new_todo, mut user): (NewTodo, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "add-multi-todos");
    Todo::create(new_todo, &mut user.conn).map(|_| basic_success())
}

/// Lets a member mark a todo they were assigned as completed.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the todo item
pub fn mark_todo_as_complete(todo_id: i32, mut user: User) -> GreaseResult<Value> {
    let todo = Todo::load(todo_id, &mut user.conn)?;
    if todo.member != user.member.member.email {
        Err(GreaseError::Forbidden(None))
    } else {
        Todo::mark_complete(todo_id, &mut user.conn).map(|_| basic_success())
    }
}

/// Send a meeting minutes as an email to the officer's list.
///
/// WARNING: This endpoint is not yet implemented. Please contact one
/// of the developers of this API if you need it done especially soon.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
pub fn send_minutes_as_email(_id: i32, user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    // TODO: implement this functionality (figure out how to compile SSL statically)
    Err(GreaseError::BadRequest(
        "emailing minutes not implemented yet.".to_owned(),
    ))
}

/// Delete a meeting's minutes.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
pub fn delete_meeting_minutes(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::delete(id, &mut user.conn).map(|_| basic_success())
}

/// Get a single uniform.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the uniform
///
/// ## Return Format:
///
/// Returns a [Uniform](crate::db::models::Uniform).
pub fn get_uniform(id: i32, mut user: User) -> GreaseResult<Value> {
    Uniform::load(id, &mut user.conn).map(|uniform| json!(uniform))
}

/// Get all of the club's uniforms.
///
/// ## Return Format:
///
/// Returns a [Uniform](crate::db::models::Uniform) ordered by name.
pub fn get_uniforms(mut user: User) -> GreaseResult<Value> {
    Uniform::load_all(&mut user.conn).map(|uniforms| json!(uniforms))
}

/// Create a new uniform.
///
/// ## Input Format:
///
/// Expects a [NewUniform](crate::db::models::NewUniform).
///
/// ## Return Format:
///
/// ```json
/// {
///     "id": integer
/// }
/// ```
///
/// Returns an object containing the id of the newly created uniform.
pub fn new_uniform((mut user, new_uniform): (User, NewUniform)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    Uniform::validate_color(&new_uniform.color)?;
    new_uniform
        .insert_returning_id(&mut user.conn)
        .map(|new_id| json!({ "id": new_id }))
}

/// Updated an existing uniform.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the uniform
///
/// ## Input Format:
///
/// Expects a [NewUniform](crate::db::models::NewUniform).
pub fn modify_uniform(
    id: i32,
    (mut user, changed_uniform): (User, NewUniform),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    Uniform::validate_color(&changed_uniform.color)?;
    Uniform::update(id, &changed_uniform, &mut user.conn).map(|_| basic_success())
}

/// Delete a uniform.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the uniform
pub fn delete_uniform(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    Uniform::delete(id, &mut user.conn).map(|_| basic_success())
}

/// Get a single semester.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the semester
///
/// ## Return Format:
///
/// Returns a [Semester](crate::db::models::Semester).
pub fn get_semester(name: String, mut user: User) -> GreaseResult<Value> {
    Semester::load(&name, &mut user.conn).map(|semester| json!(semester))
}

/// Get the current semester.
///
/// ## Return Format:
///
/// Returns a [Semester](crate::db::models::Semester).
pub fn get_current_semester(mut user: User) -> GreaseResult<Value> {
    Semester::load_current(&mut user.conn).map(|semester| json!(semester))
}

/// Get all semesters.
///
/// ## Return Format:
///
/// Returns a list of [Semester](crate::db::models::Semester)s
/// ordered by [startDate](crate::db::models::Semester#structfield.start_date).
pub fn get_semesters(mut user: User) -> GreaseResult<Value> {
    Semester::load_all(&mut user.conn).map(|semesters| json!(semesters))
}

/// Create a new semester.
///
/// ## Input Format:
///
/// Expects a [NewSemester](crate::db::models::NewSemester).
///
/// ## Return Format:
///
/// ```json
/// {
///     "name": string
/// }
/// ```
///
/// Returns an object containing the name of the new semester.
pub fn new_semester((new_semester, mut user): (NewSemester, User)) -> GreaseResult<Value> {
    Semester::create(new_semester, &mut user.conn).map(|name| json!({ "name": name }))
}

/// Set which semester is the current one.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the selected semester
pub fn set_current_semester(name: String, mut user: User) -> GreaseResult<Value> {
    Semester::set_current(&name, &mut user.conn).map(|_| basic_success())
}

/// Edit an existing semester.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the semester
///
/// ## Input Format:
///
/// Expects a [SemesterUpdate](crate::db::models::SemesterUpdate).
pub fn edit_semester(
    name: String,
    (updated_semester, mut user): (SemesterUpdate, User),
) -> GreaseResult<Value> {
    Semester::update(&name, &updated_semester, &mut user.conn).map(|_| basic_success())
}

/// Delete a semester from the site permanently.
///
/// WARNING! This is a permanent action, and cannot be undone. Make sure that
/// you know what you are doing. You must pass `confirm=true` to actually delete
/// a semester.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the semester
///
/// ## Query Parameters:
///   * confirm: boolean (*optional*) - Confirm the deletion of the semester
///
/// ## Return Format:
///
/// ```json
/// {
///     "current": string
/// }
/// ```
///
/// Returns an object containing the name of the new current semester.
pub fn delete_semester(name: String, confirm: Option<bool>, mut user: User) -> GreaseResult<Value> {
    if confirm.unwrap_or(false) {
        Err(GreaseError::BadRequest(
            "make sure to pass `confirm=true` to actually delete the semester".to_owned(),
        ))
    } else {
        Semester::delete(&name, &mut user.conn).map(|current| json!({ "current": current }))
    }
}

/// Get all permissions of the site.
///
/// ## Return Format:
///
/// Returns a list of [Permission](crate::db::models::Permission)s.
pub fn get_permissions(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<Permission>(&Permission::select_all_in_order("name", Order::Asc))
        .map(|permissions| json!(permissions))
}

/// Get all roles on the site.
///
/// ## Return Format:
///
/// Returns a list of [Role](crate::db::models::Role)s.
pub fn get_roles(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<Role>(&Role::select_all_in_order("rank", Order::Asc))
        .map(|roles| json!(roles))
}

/// Get the current officers of the club.
///
/// ## Return Format:
///
/// ```json
/// [
///     {
///         "member": Member,
///         "role": Role
///     },
///     ...
/// ]
/// ```
///
/// Returns a list of objects showing which member holds which role.
/// See [Role](crate::db::models::Role) and
/// [Member](crate::db::models::Member) for their JSON formats.
pub fn get_current_officers(mut user: User) -> GreaseResult<Value> {
    MemberRole::load_all(&mut user.conn).map(|member_role_pairs| {
        member_role_pairs
            .into_iter()
            .map(|(member, role)| {
                json!({
                    "member": member.to_json(),
                    "role": role
                })
            })
            .collect::<Vec<_>>()
            .into()
    })
}

/// Get all permissions held by a member.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///
/// ## Return Format:
///
/// ```json
/// [
///     {
///         "name": string,
///         "eventType": string?
///     },
///     ...
/// ]
/// ```
///
/// Returns a list of objects with all permissions the member has
/// and whether those permissions are for a specific event type.
pub fn member_permissions(member: String, mut user: User) -> GreaseResult<Value> {
    if &member == &user.member.member.email {
        Ok(json!(user.permissions))
    } else {
        check_for_permission!(user => "edit-permissions");
        let member = MemberForSemester::load_for_current_semester(&member, &mut user.conn)?;
        member
            .permissions(&mut user.conn)
            .map(|permissions| json!(permissions))
    }
}

/// Get all permissions held by each role.
///
/// ## Return Format:
///
/// Returns a list of [RolePermission](crate::db::models::RolePermission)s.
pub fn get_current_role_permissions(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<RolePermission>(&RolePermission::select_all())
        .map(|role_permissions| json!(role_permissions))
}

/// Award a permission (possibly for an event type) to a role.
///
/// ## Path Parameters:
///   * position: string (*required*) - The name of the position
///
/// ## Input Format:
///
/// Expects a [MemberPermission](crate::auth::MemberPermission).
pub fn add_permission_for_role(
    position: String,
    (new_permission, mut user): (MemberPermission, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-permissions");
    RolePermission::enable(
        &position,
        &new_permission.name,
        &new_permission.event_type,
        &mut user.conn,
    )
    .map(|_| basic_success())
}

/// Take away a permission (possibly for an event type) from a role.
///
/// ## Path Parameters:
///   * position: string (*required*) - The name of the position
///
/// ## Input Format:
///
/// Expects a [MemberPermission](crate::auth::MemberPermission).
pub fn remove_permission_for_role(
    position: String,
    (permission, mut user): (MemberPermission, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-permissions");
    RolePermission::disable(
        &position,
        &permission.name,
        &permission.event_type,
        &mut user.conn,
    )
    .map(|_| basic_success())
}

/// Award a member an officer position.
///
/// ## Input Format:
///
/// Expects a [MemberRole](crate::db::models::MemberRole).
pub fn add_officership((member_role, mut user): (MemberRole, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-officers");
    let given_role = user.conn.first::<Role>(
        &Role::filter(&format!("name = '{}'", &member_role.role)),
        format!("No role with name {}.", &member_role.role),
    )?;
    let member_role_pairs = MemberRole::load_all(&mut user.conn)?;
    if member_role_pairs
        .iter()
        .any(|(member, role)| role.name == member_role.role && member.email == member_role.member)
    {
        Err(GreaseError::BadRequest(format!(
            "member {} already has that position",
            &member_role.member
        )))
    } else if given_role.max_quantity > 0
        && member_role_pairs
            .iter()
            .filter(|(_member, role)| role.name == given_role.name)
            .count()
            >= given_role.max_quantity as usize
    {
        Err(GreaseError::BadRequest(format!(
            "No more officers of type {} are allowed (max of {})",
            given_role.name, given_role.max_quantity
        )))
    } else {
        member_role.insert(&mut user.conn)?;

        Ok(basic_success())
    }
}

/// Remove a member's officer position.
///
/// ## Input Format:
///
/// Expects a [MemberRole](crate::db::models::MemberRole).
pub fn remove_officership((member_role, mut user): (MemberRole, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-officers");
    user.conn.delete(
        Delete::new(MemberRole::table_name()).filter(&format!(
            "member = '{}' AND role = '{}'",
            member_role.member, member_role.role
        )),
        format!(
            "Member {} does not hold the {} position.",
            member_role.member, member_role.role
        ),
    )?;

    Ok(basic_success())
}

/// Get all of a member's transactions.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///
/// ## Return Format:
///
/// Returns a list of [Transaction](crate::db::models::Transaction)s in
/// chronological order.
pub fn get_member_transactions(email: String, mut user: User) -> GreaseResult<Value> {
    if email != user.member.member.email {
        check_for_permission!(user => "view-transactions");
    }
    Transaction::load_all_for_member(&email, &mut user.conn).map(|transactions| json!(transactions))
}

/// Add multiple transactions.
///
/// ## Input Format:
///
/// Expects a list of [Transaction](crate::db::models::Transaction)s.
pub fn add_transactions(
    (new_transactions, mut user): (Vec<NewTransaction>, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-transaction");

    user.conn.transaction(|transaction| {
        for new_transaction in new_transactions {
            new_transaction.insert(transaction)?;
        }

        Ok(basic_success())
    })
}

/// Get all transaction types.
///
/// ## Return Format:
///
/// Returns a list of [TransactionType](crate::db::models::TransactionType)s
/// ordered by name.
pub fn get_transaction_types(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<TransactionType>(&TransactionType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

/// Get a single event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Query Parameters:
///   * full: boolean (*optional*) - Whether to include uniform and attendance.
///
/// ## Return Format:
///
/// If `full = true`, then the format from
/// [to_json_full](crate::db::models::event::EventWithGig#method.to_json_full)
/// will be returned. Otherwise, the format from
/// [to_json](crate::db::models::event::EventWithGig#method.to_json)
/// will be returned.
pub fn get_fees(mut user: User) -> GreaseResult<Value> {
    Fee::load_all(&mut user.conn).map(|fees| json!(fees))
}

/// Apply a fee to all currently active semesters.
///
/// CAUTION: This endpoint may not yet work correctly, so use with caution.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the fee
pub fn apply_fee_for_all_active_members(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-transaction");
    let fee = Fee::load(&name, &mut user.conn)?;
    fee.charge_for_the_semester(&mut user.conn)?;

    Ok(basic_success())
}

/// Update the amount a fee charges when applied.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the fee
///   * amount: integer (*required*) - The new amount to charge
pub fn update_fee_amount(name: String, new_amount: i32, mut user: User) -> GreaseResult<Value> {
    Fee::update_amount(&name, new_amount, &mut user.conn).map(|_| basic_success())
}
