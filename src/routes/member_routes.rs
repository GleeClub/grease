use super::basic_success;
use crate::auth::User;
use crate::check_for_permission;
use crate::db::models::*;
use crate::error::{GreaseError, GreaseResult};
use grease_derive::Extract;
use mysql::Conn;
use serde::Deserialize;
use serde_json::{json, Value};
// use crate::db::models::member::MemberForSemester;
use std::collections::HashSet;

#[derive(Deserialize, Extract)]
pub struct LoginInfo {
    email: String,
    pass_hash: String,
}

pub fn login((form, mut conn): (LoginInfo, Conn)) -> GreaseResult<Value> {
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
    Member::load(&email, &mut user.conn).and_then(|member| {
        if details.unwrap_or(false) {
            member.to_json_full(None, &mut user.conn)
        } else if grades.unwrap_or(false) {
            let active_semester = ActiveSemester::load(
                &member.email,
                &user.member.active_semester.semester,
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
    // TODO: debug how missing "include" param leads to Some("") being provided
    let (include_class, include_club, include_inactive) = if let Some(include) =
        include.filter(|include| include.len() > 0)
    {
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

pub fn register_for_semester(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}

pub fn update_member_semester(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}

pub fn new_member(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}

pub fn update_member_profile(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}

pub fn update_member_as_officer(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}

pub fn login_as_member(member: String, mut user: User) -> GreaseResult<Value> {
    unimplemented!()
}
