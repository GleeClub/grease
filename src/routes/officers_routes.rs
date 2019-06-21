use super::basic_success;
use crate::check_for_permission;
use auth::User;
use db::models::member::{MemberForSemester, MemberPermission};
use db::models::minutes::NewMeetingMinutes;
use db::models::*;
use db::traits::{Insertable, Queryable, TableName};
use error::{GreaseError, GreaseResult};
use pinto::query_builder::{self, Order};
use serde_json::{json, Value};

pub fn get_announcement(id: i32, mut user: User) -> GreaseResult<Value> {
    Announcement::load(id, &mut user.conn).map(|announcement| json!(announcement))
}

pub fn get_announcements(all: Option<bool>, mut user: User) -> GreaseResult<Value> {
    if all.unwrap_or(false) {
        Announcement::load_all(&mut user.conn).map(|announcements| json!(announcements))
    } else {
        Announcement::load_all_for_semester(&user.member.active_semester.semester, &mut user.conn)
            .map(|announcements| json!(announcements))
    }
}

pub fn make_new_announcement(
    (mut user, new_announcement): (User, NewAnnouncement),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-announcements");
    Announcement::insert(
        &new_announcement.content,
        &user.member.member.email,
        &user.member.active_semester.semester,
        &mut user.conn,
    )
    .map(|new_id| json!({ "id": new_id }))
}

pub fn archive_announcement(announcement_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-announcements");
    Announcement::archive(announcement_id, &mut user.conn).map(|_| basic_success())
}

pub fn get_google_doc(name: String, mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load(&name, &mut user.conn).map(|doc| json!(doc))
}

pub fn get_google_docs(mut user: User) -> GreaseResult<Value> {
    GoogleDoc::load_all(&mut user.conn).map(|docs| json!(docs))
}

pub fn new_google_doc((mut user, new_doc): (User, GoogleDoc)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::insert(&new_doc, &mut user.conn).map(|id| json!({ "id": id }))
}

pub fn modify_google_doc(
    name: String,
    (mut user, changed_doc): (User, GoogleDoc),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::update(&name, &changed_doc, &mut user.conn).map(|_| basic_success())
}

pub fn delete_google_doc(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    GoogleDoc::delete(&name, &mut user.conn).map(|_| basic_success())
}

pub fn get_meeting_minutes(minutes_id: i32, mut user: User) -> GreaseResult<Value> {
    let can_view_complete_minutes = user.has_permission("view-complete-minutes", None);
    MeetingMinutes::load(minutes_id, &mut user.conn)
        .map(|minutes| minutes.to_json(can_view_complete_minutes))
}

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

pub fn modify_meeting_minutes(
    minutes_id: i32,
    (mut user, changed_minutes): (User, NewMeetingMinutes),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::update(minutes_id, &changed_minutes, &mut user.conn).map(|_| basic_success())
}

pub fn new_meeting_minutes(
    (mut user, changed_minutes): (User, NewMeetingMinutes),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::create(&changed_minutes, &mut user.conn).map(|id| json!({ "id": id }))
}

pub fn get_todos(mut user: User) -> GreaseResult<Value> {
    Todo::load_all_for_member(&user.member.member.email, &mut user.conn).map(|todos| json!(todos))
}

pub fn add_todo_for_members((new_todo, mut user): (NewTodo, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "add-multi-todos");
    Todo::create(new_todo, &mut user.conn).map(|_| basic_success())
}

pub fn mark_todo_as_complete(todo_id: i32, mut user: User) -> GreaseResult<Value> {
    let todo = Todo::load(todo_id, &mut user.conn)?;
    if todo.member != user.member.member.email {
        Err(GreaseError::Forbidden(None))
    } else {
        Todo::mark_complete(todo_id, &mut user.conn).map(|_| basic_success())
    }
}

pub fn send_minutes_as_email(_id: i32, user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    // TODO: implement this functionality (figure out how to compile SSL statically)
    Err(GreaseError::BadRequest(
        "emailing minutes not implemented yet.".to_owned(),
    ))
}

pub fn delete_meeting_minutes(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-minutes");
    MeetingMinutes::delete(id, &mut user.conn).map(|_| basic_success())
}

pub fn get_uniform(name: String, mut user: User) -> GreaseResult<Value> {
    Uniform::load(&name, &mut user.conn).map(|uniform| json!(uniform))
}

pub fn get_uniforms(mut user: User) -> GreaseResult<Value> {
    Uniform::load_all(&mut user.conn).map(|uniforms| json!(uniforms))
}

pub fn new_uniform((mut user, new_uniform): (User, Uniform)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    new_uniform.validate()?;
    new_uniform.insert(&mut user.conn).map(|_| basic_success())
}

pub fn modify_uniform(
    old_name: String,
    (mut user, changed_uniform): (User, Uniform),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    changed_uniform.validate()?;
    Uniform::update(&old_name, &changed_uniform, &mut user.conn).map(|_| basic_success())
}

pub fn delete_uniform(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-uniforms");
    Uniform::delete(&name, &mut user.conn).map(|_| basic_success())
}

pub fn get_semester(name: String, mut user: User) -> GreaseResult<Value> {
    Semester::load(&name, &mut user.conn).map(|semester| json!(semester))
}

pub fn get_semesters(mut user: User) -> GreaseResult<Value> {
    Semester::load_all(&mut user.conn).map(|semesters| json!(semesters))
}

pub fn new_semester((new_semester, mut user): (NewSemester, User)) -> GreaseResult<Value> {
    Semester::create(new_semester, &mut user.conn).map(|name| json!({ "name": name }))
}

pub fn set_current_semester(name: String, mut user: User) -> GreaseResult<Value> {
    // TODO: update officers and permissions on semester change
    Semester::set_current(&name, &mut user.conn).map(|_| basic_success())
}

pub fn edit_semester(
    name: String,
    (updated_semester, mut user): (SemesterUpdate, User),
) -> GreaseResult<Value> {
    Semester::update(&name, &updated_semester, &mut user.conn).map(|_| basic_success())
}

pub fn delete_semester(name: String, confirm: Option<bool>, mut user: User) -> GreaseResult<Value> {
    if confirm.unwrap_or(false) {
        Err(GreaseError::BadRequest(
            "make sure to pass `confirm=true` to actually delete the semester".to_owned(),
        ))
    } else {
        Semester::delete(&name, &mut user.conn).map(|current| json!({ "current": current }))
    }
}

pub fn get_permissions(mut user: User) -> GreaseResult<Value> {
    Permission::query_all_in_order(vec![("name", Order::Asc)], &mut user.conn)
        .map(|permissions| json!(permissions))
}

pub fn get_roles(mut user: User) -> GreaseResult<Value> {
    Role::query_all_in_order(vec![("rank", Order::Asc)], &mut user.conn).map(|roles| json!(roles))
}

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

pub fn get_current_role_permissions(mut user: User) -> GreaseResult<Value> {
    RolePermission::query_all_in_order(vec![("id", Order::Asc)], &mut user.conn)
        .map(|role_permissions| json!(role_permissions))
}

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

pub fn remove_permission_for_role(
    position: String,
    (new_permission, mut user): (MemberPermission, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-permissions");
    RolePermission::disable(
        &position,
        &new_permission.name,
        &new_permission.event_type,
        &mut user.conn,
    )
    .map(|_| basic_success())
}

pub fn add_officership((member_role, mut user): (MemberRole, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-officers");
    let given_role = Role::first(&format!("name = '{}'", &member_role.role), &mut user.conn, format!("no role with name '{}'", &member_role.role))?;
    let member_role_pairs = MemberRole::load_all(&mut user.conn)?;
    if member_role_pairs.iter().any(|(member, role)| role.name == member_role.role && member.email == member_role.member) {
        Err(GreaseError::BadRequest(format!("member {} already has that position", &member_role.member)))
    } else if member_role_pairs.iter().filter(|(_member, role)| role.name == given_role.name).count() >= given_role.max_quantity as usize {
        Err(GreaseError::BadRequest(format!("No more officers of type {} are allowed (max of {})", given_role.name, given_role.max_quantity)))
    } else {
        member_role.insert(&mut user.conn).map(|_| basic_success())
    }
}

pub fn remove_officership((member_role, mut user): (MemberRole, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-officers");
    let _given_role = Role::first(&format!("name = '{}'", &member_role.role), &mut user.conn, format!("no role with name '{}'", &member_role.role))?;
    let member_role_pairs = MemberRole::load_all(&mut user.conn)?;
    if !member_role_pairs.iter().any(|(member, role)| role.name == member_role.role && member.email == member_role.member) {
        Err(GreaseError::BadRequest(format!("member {} doesn't hold that position", &member_role.member)))
    } else {
        let query = query_builder::delete(MemberRole::table_name())
            .filter(&format!("member = '{}' AND role = '{}'", member_role.member, member_role.role))
            .build();
        user.conn.query(query).map_err(GreaseError::DbError)?;
        Ok(basic_success())
    }
}

pub fn get_member_transactions(email: String, mut user: User) -> GreaseResult<Value> {
    if email != user.member.member.email {
        check_for_permission!(user => "view-transactions");
    }
    Transaction::load_all_for_member(&email, &mut user.conn).map(|transactions| json!(transactions))
}

pub fn add_transactions((new_transactions, mut user): (Vec<NewTransaction>, User)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-transaction");
    let mut db_transaction = user.conn.start_transaction(false, None, None).map_err(GreaseError::DbError)?;
    for new_transaction in new_transactions {
        new_transaction.insert(&mut db_transaction)?;
    }
    Ok(basic_success())
}

pub fn get_transaction_types(mut user: User) -> GreaseResult<Value> {
    TransactionType::query_all_in_order(vec![("name", Order::Asc)], &mut user.conn).map(|types| json!(types))
}

pub fn get_fees(mut user: User) -> GreaseResult<Value> {
    Fee::load_all(&mut user.conn).map(|fees| json!(fees))
}

pub fn apply_fee_for_all_active_members(name: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-transaction");
    let fee = Fee::load(&name, &mut user.conn)?;
    fee.charge_for_the_semester(&mut user.conn)?;

    Ok(basic_success())
}

pub fn update_fee_amount(name: String, new_amount: i32, mut user: User) -> GreaseResult<Value> {
    Fee::update_amount(&name, new_amount, &mut user.conn).map(|_| basic_success())
}
