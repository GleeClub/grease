use crate::db::schema::*;
use chrono::{NaiveDate, NaiveDateTime};
use extract_derive::Extract;
use serde::{Deserialize, Serialize};

pub mod absence_request;
pub mod announcement;
pub mod attendance;
pub mod carpool;
pub mod event;
pub mod fee;
pub mod member;
pub mod misc;
pub mod semester;

// Announcement
// Attendance
// Carpool
// Event
// EventType
// Fee
// Gig
// GigRequest
// GigSong
// MemberRole
// MeetingMinutes
// Permission
// RidesIn
// Role
// RolePermission
// SectionType
// Todo
// Transaction
// TransactionType
// Uniform
// Variable

#[derive(Queryable, Serialize, Deserialize)]
pub struct AbsenceRequest {
    pub member: String,
    pub event: i32,
    pub time: NaiveDateTime,
    pub reason: String,
    pub state: AbsenceRequestState,
}

#[derive(Queryable, Insertable)]
#[table_name = "active_semester"]
pub struct ActiveSemester {
    pub member: String,
    pub semester: String,
    pub enrollment: Enrollment,
    pub section: Option<String>,
}

#[derive(Queryable, Serialize)]
pub struct Announcement {
    pub id: i32,
    pub member: Option<String>,
    pub semester: String,
    pub time: NaiveDateTime,
    pub content: String,
    pub archived: bool,
}

#[derive(Queryable)]
pub struct Attendance {
    pub member: String,
    pub event: i32,
    pub should_attend: bool,
    pub did_attend: bool,
    pub confirmed: bool,
    pub minutes_late: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Carpool {
    pub id: i32,
    pub event: i32,
    pub driver: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub semester: String,
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<String>,
}

#[derive(Queryable)]
pub struct EventType {
    pub name: String,
    pub weight: i32,
}

#[derive(Queryable)]
pub struct Fee {
    pub name: String,
    pub description: String,
    pub amount: i32,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Gig {
    pub event: i32,
    pub performance_time: NaiveDateTime,
    pub uniform: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Queryable)]
pub struct GigRequest {
    pub id: i32,
    pub time: NaiveDateTime,
    pub name: String,
    pub organization: String,
    pub event: Option<i32>,
    pub contact_name: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub start_time: NaiveDateTime,
    pub location: String,
    pub comments: Option<String>,
    pub status: GigRequestStatus,
}

#[derive(Queryable)]
pub struct GigSong {
    pub event: i32,
    pub song: i32,
    pub order: i32,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize, Extract)]
#[table_name = "google_docs"]
pub struct GoogleDoc {
    pub name: String,
    pub url: String,
}

#[derive(Queryable)]
pub struct MediaType {
    pub name: String,
    pub order: i32,
    pub storage: StorageType,
}

#[derive(Debug, Queryable, Deserialize)]
pub struct Member {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
}

#[derive(Queryable)]
pub struct MemberRole {
    pub member: String,
    pub role: String,
    pub semester: String,
}

#[derive(Queryable)]
pub struct MeetingMinutes {
    pub id: i32,
    pub name: String,
    pub date: NaiveDate,
    pub private: Option<String>,
    pub public: Option<String>,
}

#[derive(Queryable)]
pub struct Permission {
    pub name: String,
    pub description: Option<String>,
    pub type_: PermissionType,
}

#[derive(Queryable, Insertable, Debug, Serialize, Deserialize)]
#[table_name = "rides_in"]
pub struct RidesIn {
    pub member: String,
    pub carpool: i32,
}

#[derive(Queryable)]
pub struct Role {
    pub name: String,
    pub rank: i32,
    pub max_quantity: i32,
}

#[derive(Queryable)]
pub struct RolePermission {
    pub id: i32,
    pub role: String,
    pub permission: String,
    pub event_type: Option<String>,
}

#[derive(Queryable)]
pub struct SectionType {
    pub name: String,
}

#[derive(Queryable)]
pub struct Semester {
    pub name: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub gig_requirement: i32,
    pub current: bool,
}

#[derive(Deserialize, Extract)]
pub struct NewSemester {
    pub name: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

#[derive(Queryable)]
pub struct Session {
    pub member: String,
    pub key: String,
}

#[derive(Queryable)]
pub struct Song {
    pub id: i32,
    pub title: String,
    pub info: Option<String>,
    pub current: bool,
    pub key: Option<Key>,
    pub starting_pitch: Option<Key>,
    pub mode: Option<SongMode>,
}

#[derive(Queryable)]
pub struct SongLink {
    pub id: i32,
    pub song: i32,
    pub type_: String,
    pub name: String,
    pub target: String,
}

#[derive(Queryable)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub member: String,
    pub completed: bool,
}

#[derive(Queryable)]
pub struct Transaction {
    pub id: i32,
    pub member: String,
    pub time: NaiveDateTime,
    pub amount: i32,
    pub description: String,
    pub semester: Option<String>,
    pub type_: Option<String>,
    pub resolved: bool,
}

#[derive(Queryable)]
pub struct TransactionType {
    pub name: String,
}

#[derive(Queryable)]
pub struct Uniform {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable, Insertable)]
#[table_name = "variable"]
pub struct Variable {
    pub key: String,
    pub value: String,
}
