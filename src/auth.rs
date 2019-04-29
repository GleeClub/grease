use db::POOL;
use diesel::mysql::MysqlConnection;
use models::*;
use std::collections::HashMap;
use std::sync::Mutex;
use warp::filters::BoxedFilter;
use warp::{self, Filter};
use error::GreaseError;

const TOKEN_NAME: &str = "grease-session-token";

#[derive(Clone)]
pub struct User<'u> {
    member: Member,
    permissions: Vec<String>,
    conn: &'u MysqlConnection,
}

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

// TODO: lifetimes on mysql connections?
pub fn login_required() -> BoxedFilter(User,)> {
    warp::filters::cookie::cookie(TOKEN_NAME)
        .or(warp::header::<String>(TOKEN_NAME))
        .unify()
        .and_then(|token: String| {
            SESSIONS.get(&token).ok_or(GreaseError::unauthorized()).and_then(|email| {
                let conn = POOL.get().unwrap();
                match Member::load_if_exists(email, conn)? {
                    Some(member) => Ok(User {
                        member: member,
                        permissions: member.permissions(conn)?,
                        conn: conn,
                    }),
                    None => Err(GreaseError::unauthorized()),
                }
            )
        }).boxed()
}

pub fn permission_required(permission: &str) -> BoxedFilter<(User,)> {
    warp::filters::cookie::cookie(TOKEN_NAME)
        .or(warp::header::<String>(TOKEN_NAME))
        .unify()
        .and_then(|token: String| {
            SESSIONS.get(&token).ok_or(GreaseError::unauthorized()).and_then(|email| {
                let conn = POOL.get().unwrap();
                match Member::load_if_exists(email, conn)? {
                    Some(member) => {
                        let permissions = member.permissions(conn)?;
                        if permissions.contains(permission) {
                            Ok(User { member, permissions, conn })
                        } else {
                            Err(GreaseError::forbidden())
                        }

                    }),
                    None => Err(GreaseError::unauthorized()),
                }
            )
        }).boxed()
}
