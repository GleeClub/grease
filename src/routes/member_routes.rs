use super::basic_success;
use crate::auth::User;
use crate::check_for_permission;
use crate::db::*;
use crate::error::{GreaseError, GreaseResult};
use grease_derive::Extract;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Deserialize, Extract)]
pub struct LoginInfo {
    email: String,
    pass_hash: String,
}

pub fn login((form, mut conn): (LoginInfo, DbConn)) -> GreaseResult<Value> {
    if let Some(_member) = Member::check_login(&form.email, &form.pass_hash, &mut conn)? {
        if let Some(existing_session) = Session::load(&form.email, &mut conn)? {
            Err(GreaseError::AlreadyLoggedIn(existing_session.key))
        } else {
            Ok(json!({
                "token": Session::generate(&form.email, &mut conn)?
            }))
        }
    } else {
        Err(GreaseError::BadRequest(
            "login info was incorrect".to_owned(),
        ))
    }
}

pub fn logout(mut user: User) -> GreaseResult<Value> {
    Session::delete(&user.member.member.email, &mut user.conn).map(|_| basic_success())
}

pub fn get_member(
    email: String,
    grades: Option<bool>,
    details: Option<bool>,
    mut user: User,
) -> GreaseResult<Value> {
    if &email != &user.member.member.email {
        check_for_permission!(user => "view-users");
    }
    let current_semester = Semester::load_current(&mut user.conn)?;
    Member::load(&email, &mut user.conn).and_then(|member| {
        if details.unwrap_or(false) {
            member.to_json_full(None, &mut user.conn)
        } else if grades.unwrap_or(false) {
            let active_semester = ActiveSemester::load(
                &member.email,
                &current_semester.name,
                &mut user.conn,
            )?;
            if let Some(active_semester) = active_semester {
                member.to_json_with_grades(Some(active_semester), &mut user.conn)
            } else {
                Err(GreaseError::NotActiveYet(member))
            }
        } else {
            Ok(member.to_json())
        }
    })
}

pub fn get_members(
    grades: Option<bool>,
    include: Option<String>,
    mut user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "view-users");
    let current_semester = Semester::load_current(&mut user.conn)?;
    let (include_class, include_club, include_inactive) = if let Some(include) = include {
        let mut included = include.split(",").collect::<HashSet<&str>>();
        included.remove("");
        let include_class = included.remove("class");
        let include_club = included.remove("club");
        let include_inactive = included.remove("inactive");
        if included.len() > 0 {
            return Err(GreaseError::BadRequest(
                "for include param, only 'class', 'club', and 'inactive' are allowed".to_owned(),
            ));
        }

        (include_class, include_club, include_inactive)
    } else {
        (true, true, false)
    };

    Member::load_all(&mut user.conn).and_then(|members| {
        members
            .into_iter()
            .filter_map(|member| {
                let active_semester = match ActiveSemester::load(
                    &member.email,
                    &current_semester.name,
                    &mut user.conn,
                ) {
                    Ok(maybe_active_semester) => maybe_active_semester,
                    Err(error) => return Some(Err(error)),
                };
                if let Some(ref active_semester) = active_semester {
                    if !(include_class && active_semester.enrollment == Enrollment::Class)
                        && !(include_club && active_semester.enrollment == Enrollment::Club)
                    {
                        return None;
                    }
                } else if !include_inactive {
                    return None;
                }

                let json_val = if grades.unwrap_or(false) {
                    member.to_json_with_grades(active_semester, &mut user.conn)
                } else {
                    Ok(member.to_json())
                };
                Some(json_val)
            })
            .collect::<GreaseResult<Vec<_>>>()
            .map(|members| json!(members))
    })
}

pub fn new_member((new_member, mut conn): (NewMember, DbConn)) -> GreaseResult<Value> {
    Member::create(new_member, &mut conn).map(|_| basic_success())
}

pub fn register_for_semester(
    token: String,
    (form, mut conn): (RegisterForSemesterForm, DbConn),
) -> GreaseResult<Value> {
    if let Some(session) =
        conn.first_opt::<Session>(&Session::filter(&format!("`key` = '{}'", &token)))?
    {
        Member::register_for_semester(session.member, form, &mut conn).map(|_| basic_success())
    } else {
        Err(GreaseError::Unauthorized)
    }
}

pub fn mark_member_inactive_for_semester(
    member: String,
    semester: String,
    mut user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    Member::mark_inactive_for_semester(&member, &semester, &mut user.conn).map(|_| basic_success())
}

pub fn update_member_semester(
    member: String,
    semester: String,
    (update, mut user): (ActiveSemesterUpdate, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    ActiveSemester::update(&member, &semester, update, &mut user.conn).map(|_| basic_success())
}

pub fn update_member_profile((update, mut user): (NewMember, User)) -> GreaseResult<Value> {
    Member::update(&user.member.member.email, true, update, &mut user.conn).map(|_| basic_success())
}

pub fn update_member_as_officer(
    member: String,
    (update, mut user): (NewMember, User),
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    Member::update(&member, false, update, &mut user.conn).map(|_| basic_success())
}

pub fn login_as_member(member: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "switch-user");
    if member == user.member.member.email {
        Err(GreaseError::BadRequest(
            "Cannot re-login as self.".to_owned(),
        ))
    } else {
        Session::generate(&member, &mut user.conn).map(|new_key| json!({ "token": new_key }))
    }
}

pub fn delete_member(member: String, confirm: Option<bool>, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "delete-user");
    if confirm.unwrap_or(false) {
        Member::delete(&member, &mut user.conn).map(|_| basic_success())
    } else {
        Err(GreaseError::BadRequest(
            "You must pass 'confirm=true' to actually delete a member.".to_owned(),
        ))
    }
}
