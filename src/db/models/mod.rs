use chrono::{NaiveDateTime, NaiveDate};
use schema::*;

#[derive(Queryable, Identifiable)]
#[table_name = "absence_request"]
#[primary_key(member, event)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Event, foreign_key = "event")]
pub struct AbsenceRequest {
    pub member: String,
    pub event: i32,
    pub time: NaiveDateTime,
    pub reason: String,
    pub state: AbsenceRequestState,
}

#[derive(Queryable, Identifiable)]
#[table_name = "active_semester"]
#[primary_key(member, semester, choir)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct ActiveSemester {
    pub member: String,
    pub semester: i32,
    pub choir: String,
    pub enrollment: Enrollment,
    pub section: Option<i32>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "announcement"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Choir, foreign_key = "choir")]
#[belongs_to(Semester, foreign_key = "semester")]
pub struct Announcement {
    pub id: i32,
    pub choir: String,
    pub member: String,
    pub semester: i32,
    pub time: NaiveDateTime,
    pub content: String,
    pub archived: bool,
}

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
#[table_name = "carpool"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "driver")]
#[belongs_to(Event, foreign_key = "event")]
pub struct Carpool {
    pub id: i32,
    pub event: i32,
    pub driver: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "choir"]
#[primary_key(name)]
pub struct Choir {
    pub name: String,
	pub officer_email_list: String,
	pub member_email_list: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "event"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(EventType, foreign_key = "type")]
#[belongs_to(SectionType, foreign_key = "section")]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub choir: String,
    pub semester: String,
    pub type: i32,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<i32>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "event_type"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct EventType {
    pub id: i32,
    pub name: String,
    pub choir: String,
    pub weight: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "fee"]
#[primary_key(name, choir)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Fee {
    pub name: String,
    pub choir: String,
    pub amount: i32,
}

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
#[table_name = "google_docs"]
#[primary_key(name, choir)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct GoogleDoc {
    pub name: String,
    pub choir: String,
    pub url: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "media_type"]
#[primary_key(name)]
pub struct MediaType {
    pub name: String,
    pub order: i32,
    pub storage: StorageType,
}

#[derive(Queryable, Identifiable)]
#[table_name = "member"]
#[primary_key(email)]
pub struct Member {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
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

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
#[table_name = "minutes"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct MeetingMinutes {
    pub id: i32,
    pub choir: String,
    pub name: String,
    pub date: NaiveDate,
    pub private: Option<String>,
    pub public: Option<String>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "permission"]
#[primary_key(name)]
pub struct Permission {
    pub name: String,
    pub description: Option<String>,
    pub type: PermissionType,
}

#[derive(Queryable, Identifiable)]
#[table_name = "outfit"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Outfit {
    pub id: i32,
    pub name: String,
    pub choir: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "outfit"]
#[primary_key(id)]
#[belongs_to(Outfit, foreign_key = "outfit")]
#[belongs_to(Member, foreign_key = "member")]
pub struct OutfitBorrow {
    pub outfit: i32,
    pub member: String,
    pub status: BorrowStatus,
}

#[derive(Queryable, Identifiable)]
#[table_name = "permission"]
#[primary_key(name)]
pub struct Permission {
    pub name: String,
	pub description: Option<String>,
	pub type: PermissionType,
}

#[derive(Queryable, Identifiable)]
#[table_name = "ridesin"]
#[primary_key(member, carpool)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Carpool, foreign_key = "carpool")]
pub struct RidesIn {
    pub member: String,
    pub carpool: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "role"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Role {
    pub id: i32,
    pub name: Option<String>,
    pub choir: String,
    pub rank: i32,
    pub max_quantity: i32,
}

#[derive(Queryable, Identifiable)]
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

#[derive(Queryable, Identifiable)]
#[table_name = "section_type"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct SectionType {
    pub id: i32,
	pub name: String,
	pub choir: Option<String>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "semester"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Semester {
	pub id: i32,
    pub name: String,
    pub choir: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub gig_requirement: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "semester"]
#[primary_key(member)]
#[belongs_to(Member, foreign_key = "member")]
pub struct Session {
    pub member: String,
    pub key: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "song"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Song {
    pub id: i32,
    pub choir: String,
    pub title: String,
    pub info: Option<String>,
    pub current: bool,
    pub key: Option<Key>,
    pub starting_pitch: Option<Key>,
    pub mode: Option<SongMode>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "song_link"]
#[primary_key(id)]
#[belongs_to(Song, foreign_key = "song")]
#[belongs_to(MediaType, foreign_key = "type")]
pub struct SongLink {
    pub id: i32,
    pub song: i32,
    pub type: String,
    pub name: String,
    pub target: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "todo"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
pub struct Todo {
    pub id: i32,
	pub text: String,
	pub member: String,
	pub completed: bool,
}

#[derive(Queryable, Identifiable)]
#[table_name = "transaction"]
#[primary_key(id)]
#[belongs_to(Member, foreign_key = "member")]
#[belongs_to(Choir, foreign_key = "choir")]
#[belongs_to(Semester, foreign_key = "semester")]
#[belongs_to(TransactionType, foreign_key = "type")]
pub struct Transaction {
    pub id: i32,
    pub member: String,
    pub choir: String,
    pub time: NaiveDateTime,
    pub amount: i32,
    pub description: String,
    pub semester: Option<i32>,
    pub type: Option<i32>,
    pub resolved: bool,
}

#[derive(Queryable, Identifiable)]
#[table_name = "transaction_type"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct TransactionType {
    pub id: i32,
    pub name: String,
    pub choir: String,
}

#[derive(Queryable, Identifiable)]
#[table_name = "uniform"]
#[primary_key(id)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Uniform {
    pub id: i32,
    pub name: String,
    pub choir: String,
    pub description: Option<String>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "variable"]
#[primary_key(choir, key)]
#[belongs_to(Choir, foreign_key = "choir")]
pub struct Variable {
    pub choir: String,
    pub key: String,
    pub value: String,
}
