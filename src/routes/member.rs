use crate::auth::User;
use crate::db::models::*;
use crate::db::models::member::MemberPermission;
use crate::error::{GreaseError, GreaseResult};
use app_route::AppRoute;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use extract_derive::Extract;
use diesel::MysqlConnection;
use super::{OptionalIdQuery, OptionalEmailQuery};
use crate::check_for_permission;

#[derive(Deserialize, Extract)]
pub struct LoginInfo {
    email: String,
    pass_hash: String,
}

#[derive(AppRoute, Debug)]
#[route("/login")]
pub struct LoginRequest {}

pub fn login(_req: LoginRequest, (login_info, conn): (LoginInfo, MysqlConnection)) -> GreaseResult<Value> {
    return Err(GreaseError::BadRequest("checking login...".to_owned()));
    if let Some(_member) = Member::check_login(&login_info.email, &login_info.pass_hash, &conn)? {
        if let Some(existing_session) = Session::load(&login_info.email, &conn)? {
            Err(GreaseError::AlreadyLoggedIn(existing_session.key))
        } else {
            Ok(json!({
                "token": Session::generate(&login_info.email, &conn)?
            }))
        }
    } else {
        Err(GreaseError::Unauthorized)
    }
}

#[derive(AppRoute, Debug)]
#[route("/logout")]
pub struct LogoutRequest {}

pub fn logout(_req: LogoutRequest, user: User) -> GreaseResult<Value> {
    Session::delete(&user.member.member.email, &user.conn)?;

    Ok(json!({
        "message": "OK"
    }))
}

#[derive(AppRoute, Debug)]
#[route("/members")]
pub struct MembersRequest {
    #[query]
    pub query: OptionalMembersQuery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalMembersQuery {
    email: Option<String>,
    details: Option<bool>,
}

pub fn get_members(req: MembersRequest, user: User) -> GreaseResult<Value> {
    if let Some(ref email) = &req.query.email {
        if email != &user.member.member.email {
            check_for_permission!(user => "view-users");
        }
        Member::load(&email, &user.conn).and_then(|member| if req.query.details.unwrap_or(false) {
            member.to_json_full(&user.conn)
        } else {
            Ok(member.to_json())
        })
    } else {
        check_for_permission!(user => "view-users");
        Member::load_all(&user.conn).and_then(|members| {
            Ok(json!(members
                .into_iter()
                .map(|member| if req.query.details.unwrap_or(false) {
                    member.to_json_full(&user.conn)
                } else {
                    Ok(member.to_json())
                })
                .collect::<GreaseResult<Vec<_>>>()?))
        })
    }
}
