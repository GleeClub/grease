use chrono::{NaiveDateTime, NaiveDate};
use crate::db::schema::*;
use serde::{Serialize, Deserialize};

pub mod absence_request;
pub mod active_semester;
pub mod announcement;
pub mod attendance;
pub mod carpool;
pub mod event;
pub mod fee;
pub mod google_docs;
pub mod member;
pub mod semester;
pub mod misc;

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
// Outfit
// OutfitBorrow
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
#[table_name = "absence_request"]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Event, foreign_key = "event")]
#[primary_key(member, event)]
pub struct AbsenceRequest {
    pub member: String,
    pub event: i32,
    pub time: NaiveDateTime,
    pub reason: String,
    pub state: AbsenceRequestState,
}

#[derive(Queryable)]
#[table_name = "active_semester"]
#[primary_key(member, semester)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Semester, foreign_key = "semester")]
pub struct ActiveSemester {
    pub member: String,
    pub semester: i32,
    pub enrollment: Enrollment,
    pub section: Option<i32>,
}

#[derive(Queryable)]
#[table_name = "announcement"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Semester, foreign_key = "semester")]
pub struct Announcement {
    pub id: i32,
    pub member: String,
    pub semester: i32,
    pub time: NaiveDateTime,
    pub content: String,
    pub archived: bool,
}

#[derive(Queryable)]
#[table_name = "attendance"]
#[primary_key(member, event)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Event, foreign_key = "event")]
pub struct Attendance {
    pub member: String,
    pub event: i32,
    pub should_attend: bool,
    pub did_attend: Option<bool>,
    pub confirmed: bool,
    pub minutes_late: i32,
}

#[derive(Queryable)]
#[table_name = "carpool"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "driver")]
#[belongs_to(Event, foreign_key = "event")]
pub struct Carpool {
    pub id: i32,
    pub event: i32,
    pub driver: String,
}

#[derive(Queryable)]
#[table_name = "event"]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(EventType, foreign_key = "type")]
#[belongs_to(SectionType, foreign_key = "section")]
#[primary_key(id)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub semester: String,
    pub type_: i32,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<i32>,
}

#[derive(Queryable)]
#[table_name = "event_type"]
#[primary_key(id)]
pub struct EventType {
    pub id: i32,
    pub name: String,
    pub weight: i32,
}

#[derive(Queryable)]
#[table_name = "fee"]
#[primary_key(name)]
pub struct Fee {
    pub name: String,
    pub amount: i32,
}

#[derive(Queryable)]
#[table_name = "gig"]
#[primary_key(event)]
#[belongs_to(Event, foreign_key = "event")]
#[belongs_to(Uniform, foreign_key = "uniform")]
pub struct Gig {
    pub event: i32,
    pub performance_time: NaiveDateTime,
    pub uniform: i32,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Queryable)]
#[table_name = "gig_request"]
#[primary_key(id)]
#[belongs_to(Event, foreign_key = "event")]
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
#[table_name = "gig_song"]
#[primary_key(id)]
#[belongs_to(Event, foreign_key = "event")]
#[belongs_to(Song, foreign_key = "song")]
pub struct GigSong {
    pub id: i32,
    pub event: i32,
    pub song: i32,
    pub order: i32,
}

#[derive(Queryable)]
#[table_name = "google_docs"]
#[primary_key(name)]
pub struct GoogleDoc {
    pub name: String,
    pub url: String,
}

#[derive(Queryable)]
#[table_name = "media_type"]
#[primary_key(name)]
pub struct MediaType {
    pub name: String,
    pub order: i32,
    pub storage: StorageType,
}

#[derive(Queryable)]
#[table_name = "member"]
#[primary_key(email)]
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
#[table_name = "member_role"]
#[primary_key(member, role, semester)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Role, foreign_key = "role")]
#[belongs_to(Semester, foreign_key = "semester")]
pub struct MemberRole {
    pub member: String,
    pub role: i32,
    pub semester: i32,
}

#[derive(Queryable)]
#[table_name = "minutes"]
#[primary_key(id)]
pub struct MeetingMinutes {
    pub id: i32,
    pub name: String,
    pub date: NaiveDate,
    pub private: Option<String>,
    pub public: Option<String>,
}

#[derive(Queryable)]
#[table_name = "outfit"]
#[primary_key(id)]
pub struct Outfit {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
#[table_name = "outfit"]
#[primary_key(outfit)]
#[belongs_to(Outfit, foreign_key = "outfit")]
#[belongs_to(Member, foreign_key = "member")]
pub struct OutfitBorrow {
    pub outfit: i32,
    pub member: String,
    pub status: BorrowStatus,
}

#[derive(Queryable)]
#[table_name = "permission"]
#[primary_key(name)]
pub struct Permission {
    pub name: String,
	pub description: Option<String>,
	pub type_: PermissionType,
}

#[derive(Queryable)]
#[table_name = "rides_in"]
#[primary_key(member, carpool)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Carpool, foreign_key = "carpool")]
pub struct RidesIn {
    pub member: String,
    pub carpool: i32,
}

#[derive(Queryable)]
#[table_name = "role"]
#[primary_key(id)]
pub struct Role {
    pub id: i32,
    pub name: Option<String>,
    pub rank: i32,
    pub max_quantity: i32,
}

#[derive(Queryable)]
#[table_name = "role_permission"]
#[primary_key(id)]
#[belongs_to(Role, foreign_key = "role")]
#[belongs_to(Permission, foreign_key = "permission")]
#[belongs_to(EventType, foreign_key = "event_type")]
pub struct RolePermission {
    pub id: i32,
    pub role: i32,
    pub permission: String,
    pub event_type: Option<i32>,
}

#[derive(Queryable)]
#[table_name = "section_type"]
#[primary_key(id)]
pub struct SectionType {
    pub id: i32,
	pub name: String,
}

#[derive(Queryable)]
#[table_name = "semester"]
#[primary_key(id)]
pub struct Semester {
	pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub gig_requirement: i32,
}

#[derive(Queryable)]
#[table_name = "semester"]
#[primary_key(member)]
#[belongs_to(Member, foreign_key = "member")]
pub struct Session {
    pub member: String,
    pub key: String,
}

#[derive(Queryable)]
#[table_name = "song"]
#[primary_key(id)]
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
#[table_name = "song_link"]
#[primary_key(id)]
#[belongs_to(Song, foreign_key = "song")]
#[belongs_to(MediaType, foreign_key = "type")]
pub struct SongLink {
    pub id: i32,
    pub song: i32,
    pub type_: String,
    pub name: String,
    pub target: String,
}

#[derive(Queryable)]
#[table_name = "todo"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
pub struct Todo {
    pub id: i32,
	pub text: String,
	pub member: String,
	pub completed: bool,
}

#[derive(Queryable)]
#[table_name = "transaction"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(TransactionType, foreign_key = "type")]
pub struct Transaction {
    pub id: i32,
    pub member: String,
    pub time: NaiveDateTime,
    pub amount: i32,
    pub description: String,
    pub semester: Option<i32>,
    pub type_: Option<i32>,
    pub resolved: bool,
}

#[derive(Queryable)]
#[table_name = "transaction_type"]
#[primary_key(id)]
pub struct TransactionType {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable)]
#[table_name = "uniform"]
#[primary_key(id)]
pub struct Uniform {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Queryable)]
#[table_name = "variable"]
#[primary_key(key)]
pub struct Variable {
    pub key: String,
    pub value: String,
}
