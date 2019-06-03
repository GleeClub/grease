use app_route::AppRoute;
use diesel::mysql::MysqlConnection;
use crate::error::{GreaseError, GreaseResult};
use crate::auth::User;
use crate::extract::Extract;
use serde_json::Value;
use crate::db::models::*;
use serde::{Serialize, Deserialize};

macro_rules! check_for_permission {
    ($user:expr => $permission:expr) => {
        if !$user.has_permission($permission) {
            return Err(GreaseError::Forbidden($permission.to_owned()));
        }
    }
}

macro_rules! handle_routes {
    ($request:expr, $uri:expr, $given_method:expr =>
        [ $method:ident $route:ty => $handler:ident, $($methods:ident $routes:ty => $handlers:ident, )* ] ) => {
        {
            if $given_method == stringify!($method) { 
                if let Some(data) = $uri.parse::<$route>().or(format!("{}?", $uri).parse::<$route>()).ok() {
                    return $handler(data, Extract::extract(&$request)?);
                }
            }
            handle_routes!($request, $uri, $given_method => [ $($methods $routes => $handlers, )* ])
        }
    };
    ($request:expr, $uri:expr, $given_method:expr => [ ]) => {
        Err(GreaseError::NotFound)
    };
}

pub fn handle_request(request: cgi::Request) -> GreaseResult<Value> {
    handle_routes!(request, request.uri().to_string(), request.method().to_string() => [
        GET MembersRequest     => get_members,
        GET EventsRequest      => get_events,
        GET GetVariableRequest => get_variable,
        GET SetVariableRequest => set_variable,
    ])
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalEmailQuery {
    email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionalIdQuery {
    id: Option<i32>,
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/members")]
pub struct MembersRequest {
    #[query]
    pub query: OptionalEmailQuery,
}

pub fn get_members(req: MembersRequest, user: User) -> GreaseResult<Value> {
    if let Some(email) = req.query.email {
        Member::load(&email, &user.conn).map(|member| member.into())
    } else {
        Member::load_all(&user.conn).map(|members| members.into())
    }
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/events")]
pub struct EventsRequest {
    #[query]
    pub query: OptionalIdQuery,
}

pub fn get_events(req: EventsRequest, user: User) -> GreaseResult<Value> {
    if let Some(event_id) = req.query.id {
        Event::load(event_id, &user.conn).map(|event| event.into())
    } else {
        Event::load_all(&user.conn).map(|events| events.into())
    }
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/variables/:key")]
pub struct GetVariableRequest {
    pub key: String,
}

pub fn get_variable(req: GetVariableRequest, user: User) -> GreaseResult<Value> {
    Variable::load(&req.key, &user.conn).map(|var| var.into())
}

#[derive(AppRoute, Debug, PartialEq)]
#[route("/variables/:key/:value")]
pub struct SetVariableRequest {
    pub key: String,
    pub value: String,
}

pub fn set_variable(req: SetVariableRequest, user: User) -> GreaseResult<Value> {
    Variable::set(&req.key, &req.value, &user.conn).map(|old_val| old_val.into())
}

// AbsenceRequest
// ActiveSemester
// Announcement
// Attendance
// Carpool
// Event
// EventType
// Fee
// Gig
// GigRequest
// GigSong
// GoogleDoc
// MediaType
// Member
// MemberRole
// MeetingMinutes
// Outfit
// OutfitBorrow
// Permission
// RidesIn
// Role
// RolePermission
// SectionType
// Semester
// Session
// Song
// SongLink
// Todo
// Transaction
// TransactionType
// Uniform
// Variable
