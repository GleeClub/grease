//! All officer-focused routes.

use super::basic_success;
use crate::check_for_permission;
use auth::*;
use db::*;
use error::*;
use serde_json::{json, Value};

/// Get a single announcement.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the announcement
///
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-announcements" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-announcements" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns an [GoogleDoc](crate::db::models::GoogleDoc).
pub fn get_google_doc(name: String, mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load(&name, &mut user.conn).map(|doc| json!(doc))
}

/// Get all of the Google Docs.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [GoogleDoc](crate::db::models::GoogleDoc)s.
pub fn get_google_docs(mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load_all(&mut user.conn).map(|docs| json!(docs))
}

/// Create a new Google Doc.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-links" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-links" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-links" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in. To view the officer's version of meeting
/// minutes, the user needs to be able to "view-complete-minutes" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// ```json
/// [
///     {
///         "id": integer,
///         "name": string,
///         "date": date
///     },
///     ...
/// ]
/// ```
///
/// Returns a list of simplified [MeetingMinutes](crate::db::models::MeetingMinutes)
/// ordered chronologically and then alphabetically by title.
pub fn get_all_meeting_minutes(mut user: User) -> GreaseResult<Value> {
    MeetingMinutes::load_all(&mut user.conn).map(|all_minutes| {
        all_minutes
            .into_iter()
            .map(|minutes| {
                json!({
                    "id": minutes.id,
                    "name": minutes.name,
                    "date": minutes.date
                })
            })
            .collect::<Vec<_>>()
            .into()
    })
}

/// Modify an existing meeting's minutes.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-minutes" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-minutes" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "add-multi-todo" generally.
///
/// ## Input Format:
///
/// Expects a [NewTodo](crate::db::models::NewTodo).
pub fn add_todo_for_members((new_todo, mut user): (NewTodo, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "add-multi-todo");
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
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "view-complete-minutes" generally.
pub fn send_minutes_as_email(minutes_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "view-complete-minutes");
    let minutes = MeetingMinutes::load(minutes_id, &mut user.conn)?;
    let date = minutes.date.format("%B %_d, %Y");
    let subject = format!("Notes from the Officer Meeting on {}", date);
    let content = format!(
        "Notes from the meeting \"{}\" on \"{}\":\n\n{}\n",
        minutes.name, date,
        minutes.private.or(minutes.public).ok_or(GreaseError::BadRequest(format!(
            "Both the private and public versions of the meeting with id {} are empty, so no email was sent.", minutes_id)))?,
    );
    let officer_email = Variable::load("admin_list", &mut user.conn)?
        .ok_or(GreaseError::ServerError(
            "The officer's email list was not set under to `admin_list` variable.".to_owned(),
        ))?
        .value;
    let email = crate::util::Email {
        from_name: "Glee Club Officers",
        from_address: &officer_email,
        to_name: "Glee Club Officers",
        to_address: &officer_email,
        subject: &subject,
        content: &content,
    };

    email.send().map(|_| basic_success())
}

/// Delete a meeting's minutes.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the meeting minutes
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-minutes" generally.
pub fn delete_meeting_minutes(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::delete(id, &mut user.conn).map(|_| basic_success())
}

/// Get a single uniform.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the uniform
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a [Uniform](crate::db::models::Uniform).
pub fn get_uniform(id: i32, mut user: User) -> GreaseResult<Value> {
    Uniform::load(id, &mut user.conn).map(|uniform| json!(uniform))
}

/// Get all of the club's uniforms.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a [Uniform](crate::db::models::Uniform) ordered by name.
pub fn get_uniforms(mut user: User) -> GreaseResult<Value> {
    Uniform::load_all(&mut user.conn).map(|uniforms| json!(uniforms))
}

/// Create a new uniform.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-uniforms" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-uniforms" generally.
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
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-uniforms" generally.
pub fn delete_uniform(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    Uniform::delete(id, &mut user.conn).map(|_| basic_success())
}

/// Get a single semester.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the semester
///
/// ## Required Permissions:
///
/// The user must be logged in.
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
pub fn get_current_semester() -> GreaseResult<Value> {
    let mut conn = connect_to_db()?;
    Semester::load_current(&mut conn).map(|semester| json!(semester))
}

/// Get all semesters.
///
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-semester" generally.
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
    check_for_permission!(user => "edit-semester");
    Semester::create(new_semester, &mut user.conn).map(|name| json!({ "name": name }))
}

/// Set which semester is the current one.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the selected semester
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-semester" generally.
pub fn set_current_semester(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-semester");
    Semester::set_current(&name, &mut user.conn).map(|_| basic_success())
}

/// Edit an existing semester.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the semester
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-semester" generally.
///
/// ## Input Format:
///
/// Expects a [SemesterUpdate](crate::db::models::SemesterUpdate).
pub fn edit_semester(
    name: String,
    (updated_semester, mut user): (NewSemester, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-semester");
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-semester" generally.
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
    check_for_permission!(user => "edit-semester");
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
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in.
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
/// ## Required Permissions:
///
/// The user must be logged in. If they are checking someone else's permissions,
/// they need to be able to "edit-permissions" generally.
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
        let member = Member::load(&member, &mut user.conn)?;
        member
            .permissions(&mut user.conn)
            .map(|permissions| json!(permissions))
    }
}

/// Get all permissions held by each role.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-permissions" generally.
///
/// ## Return Format:
///
/// Returns a list of [RolePermission](crate::db::models::RolePermission)s.
pub fn get_current_role_permissions(mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-permissions");
    user.conn
        .load::<RolePermission>(&RolePermission::select_all())
        .map(|role_permissions| json!(role_permissions))
}

/// Award a permission (possibly for an event type) to a role.
///
/// ## Path Parameters:
///   * position: string (*required*) - The name of the position
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-permissions" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-permissions" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-officers" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-officers" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in. If they are viewing another member's transactions,
/// they need to be able to "view-transactions" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-transaction" generally.
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
/// ## Required Permissions:
///
/// The user must be logged in.
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

/// Get all the types of fees in the club.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [Fee](crate::db::models::Fee)s.
pub fn get_fees(mut user: User) -> GreaseResult<Value> {
    Fee::load_all(&mut user.conn).map(|fees| json!(fees))
}

/// Apply a fee to all currently active semesters.
///
/// CAUTION: This endpoint may not yet work correctly, so use with caution.
///
/// ## Path Parameters:
///   * name: string (*required*) - The name of the fee
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-transaction" generally.
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
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-transaction" generally.
pub fn update_fee_amount(name: String, new_amount: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-transaction");
    Fee::update_amount(&name, new_amount, &mut user.conn).map(|_| basic_success())
}
