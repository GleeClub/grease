use chrono::{NaiveDate, NaiveDateTime};
use grease_derive::*;
use mysql::prelude::ToValue;
use mysql_enum::MysqlEnum;
use serde::{Deserialize, Deserializer, Serialize};
use strum_macros::{Display, EnumString};

pub mod absence_request;
pub mod attendance;
pub mod carpool;
pub mod event;
pub mod member;
pub mod minutes;
pub mod misc;
pub mod semester;
pub mod song;
pub mod transaction;

// CREATE TABLE member (
//   email varchar(50) NOT NULL PRIMARY KEY,
//   first_name varchar(25) NOT NULL,
//   preferred_name varchar(25) DEFAULT NULL,
//   last_name varchar(25) NOT NULL,
//   pass_hash varchar(64) NOT NULL,
//   phone_number varchar(16) NOT NULL,
//   picture varchar(255) DEFAULT NULL,
//   passengers int NOT NULL DEFAULT '0',
//   location varchar(50) NOT NULL,
//   about varchar(500) DEFAULT NULL,
//   major varchar(50) DEFAULT NULL,
//   minor varchar(50) DEFAULT NULL,
//   hometown varchar(50) DEFAULT NULL,
//   arrived_at_tech int DEFAULT NULL, -- year
//   gateway_drug varchar(500) DEFAULT NULL,
//   conflicts varchar(500) DEFAULT NULL,
//   dietary_restrictions varchar(500) DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable, Debug)]
#[table_name = "member"]
pub struct Member {
    pub email: String,
    pub first_name: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub about: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub major: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub minor: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub gateway_drug: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub conflicts: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub dietary_restrictions: Option<String>,
}

// CREATE TABLE semester (
//   name varchar(32) NOT NULL PRIMARY KEY,
//   start_date datetime NOT NULL,
//   end_date datetime NOT NULL,
//   gig_requirement int NOT NULL DEFAULT '5',
//   current boolean NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable)]
#[table_name = "semester"]
pub struct Semester {
    pub name: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub gig_requirement: i32,
    pub current: bool,
}

#[derive(TableName, Deserialize, FieldNames, Extract)]
#[table_name = "semester"]
pub struct SemesterUpdate {
    pub name: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub gig_requirement: i32,
    pub current: bool,
}

#[derive(TableName, Deserialize, FieldNames, Insertable, Extract)]
#[table_name = "semester"]
pub struct NewSemester {
    pub name: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

// CREATE TABLE role (
//   name varchar(20) NOT NULL PRIMARY KEY,
//   `rank` int NOT NULL,
//   max_quantity int NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "role"]
pub struct Role {
    pub name: String,
    pub rank: i32,
    pub max_quantity: i32,
}

// CREATE TABLE member_role (
//   member varchar(50) NOT NULL,
//   role varchar(20) NOT NULL,

//   PRIMARY KEY (member, role),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable, Extract)]
#[table_name = "member_role"]
pub struct MemberRole {
    pub member: String,
    pub role: String,
}

// CREATE TABLE section_type (
//   name varchar(20) NOT NULL PRIMARY KEY
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "section_type"]
pub struct SectionType {
    pub name: String,
}

// CREATE TABLE event_type (
//   name varchar(32) NOT NULL PRIMARY KEY,
//   weight int NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "event_type"]
pub struct EventType {
    pub name: String,
    pub weight: i32,
}

// CREATE TABLE event (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   name varchar(64) NOT NULL,
//   semester varchar(32) NOT NULL,
//   `type` varchar(32) NOT NULL,
//   call_time datetime NOT NULL,
//   release_time datetime DEFAULT NULL,
//   points int NOT NULL,
//   comments text DEFAULT NULL,
//   location varchar(255) DEFAULT NULL,
//   gig_count boolean NOT NULL DEFAULT '1',
//   default_attend boolean NOT NULL DEFAULT '1',
//   section varchar(20) DEFAULT NULL,

//   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE ON DELETE CASCADE,
//   FOREIGN KEY (`type`) REFERENCES event_type (name) ON UPDATE CASCADE ON DELETE CASCADE,
//   FOREIGN KEY (section) REFERENCES section_type (name) ON UPDATE CASCADE ON DELETE SET NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Debug)]
#[table_name = "event"]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub comments: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub section: Option<String>,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract)]
#[table_name = "event"]
pub struct NewEvent {
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub comments: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub location: Option<String>,
    #[serde(default)]
    pub gig_count: Option<bool>,
    pub default_attend: bool,
    pub repeat: String,
    pub repeat_until: Option<NaiveDate>,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract, Debug)]
#[table_name = "event"]
pub struct EventUpdate {
    // event fields
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub comments: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub section: Option<String>,
    // gig fields
    pub performance_time: Option<NaiveDateTime>,
    pub uniform: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_name: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_email: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: Option<bool>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub description: Option<String>,
}

// CREATE TABLE absence_request (
//   member varchar(50) NOT NULL,
//   event int NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   reason varchar(500) NOT NULL,
//   state enum('pending', 'approved', 'denied') NOT NULL DEFAULT 'pending',

//   PRIMARY KEY (member, event),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "absence_request"]
pub struct AbsenceRequest {
    pub member: String,
    pub event: i32,
    pub time: NaiveDateTime,
    pub reason: String,
    pub state: AbsenceRequestState,
}

#[derive(Deserialize, Extract)]
pub struct NewAbsenceRequest {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum AbsenceRequestState {
    #[strum(serialize = "approved")]
    Approved,
    #[strum(serialize = "denied")]
    Denied,
    #[strum(serialize = "pending")]
    Pending,
}

// CREATE TABLE active_semester (
//   member varchar(50) NOT NULL,
//   semester varchar(32) NOT NULL,
//   enrollment enum('class', 'club') NOT NULL DEFAULT 'club',
//   section varchar(20) DEFAULT NULL,

//   PRIMARY KEY (member, semester),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (section) REFERENCES section_type (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable)]
#[table_name = "active_semester"]
pub struct ActiveSemester {
    pub member: String,
    pub semester: String,
    pub enrollment: Enrollment,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub section: Option<String>,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum Enrollment {
    #[strum(serialize = "class")]
    Class,
    #[strum(serialize = "club")]
    Club,
}

// CREATE TABLE announcement (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   member varchar(50) DEFAULT NULL,
//   semester varchar(32) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   content longtext NOT NULL,
//   archived bool NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE SET NULL ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "announcement"]
pub struct Announcement {
    pub id: i32,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub member: Option<String>,
    pub semester: String,
    pub time: NaiveDateTime,
    pub content: String,
    pub archived: bool,
}

#[derive(Deserialize, Extract)]
pub struct NewAnnouncement {
    pub content: String,
}

// CREATE TABLE attendance (
//   member varchar(50) NOT NULL,
//   event int NOT NULL,
//   should_attend boolean NOT NULL DEFAULT '1',
//   did_attend boolean NOT NULL DEFAULT '0', -- TODO: null or not if an event hasn't passed
//   confirmed boolean NOT NULL DEFAULT '0',
//   minutes_late int NOT NULL DEFAULT '0',

//   PRIMARY KEY (member, event),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Debug)]
#[table_name = "attendance"]
pub struct Attendance {
    pub member: String,
    pub event: i32,
    pub should_attend: bool,
    pub did_attend: bool,
    pub confirmed: bool,
    pub minutes_late: i32,
}

// CREATE TABLE carpool (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   event int NOT NULL,
//   driver varchar(50) NOT NULL,

//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (driver) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "carpool"]
pub struct Carpool {
    pub id: i32,
    pub event: i32,
    pub driver: String,
}

#[derive(Deserialize, TableName, Insertable, Extract)]
#[table_name = "carpool"]
pub struct NewCarpool {
    pub event: i32,
    pub driver: String,
}

#[derive(Debug, Deserialize, Extract)]
pub struct UpdatedCarpool {
    pub id: Option<i32>,
    pub driver: String,
    pub passengers: Vec<String>,
}

// CREATE TABLE fee (
//   name varchar(16) NOT NULL PRIMARY KEY,
//   description varchar(40) NOT NULL PRIMARY KEY,
//   amount int NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "fee"]
pub struct Fee {
    pub name: String,
    pub description: String,
    pub amount: i32,
}

// CREATE TABLE google_docs (
//   name varchar(40) NOT NULL PRIMARY KEY,
//   url varchar(255) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable, Extract)]
#[table_name = "google_docs"]
pub struct GoogleDoc {
    pub name: String,
    pub url: String,
}

// CREATE TABLE uniform (
//   name varchar(32) NOT NULL PRIMARY KEY,
//   color varchar(4) DEFAULT NULL,
//   description text DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable, Extract)]
#[table_name = "uniform"]
pub struct Uniform {
    pub name: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub color: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub description: Option<String>,
}

// CREATE TABLE gig (
//   event int NOT NULL PRIMARY KEY,
//   performance_time datetime NOT NULL,
//   uniform varchar(32) NOT NULL,
//   contact_name varchar(50) DEFAULT NULL,
//   contact_email varchar(50) DEFAULT NULL,
//   contact_phone varchar(16) DEFAULT NULL,
//   price int DEFAULT NULL,
//   public boolean NOT NULL DEFAULT '0',
//   summary text DEFAULT NULL,
//   description text DEFAULT NULL,

//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (uniform) REFERENCES uniform (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "gig"]
pub struct Gig {
    pub event: i32,
    pub performance_time: NaiveDateTime,
    pub uniform: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_name: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_email: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub description: Option<String>,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract)]
#[table_name = "gig"]
pub struct NewGig {
    pub performance_time: NaiveDateTime,
    pub uniform: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_name: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_email: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub description: Option<String>,
}

// CREATE TABLE gig_request (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   name varchar(255) NOT NULL,
//   organization varchar(255) NOT NULL,
//   event int DEFAULT NULL,
//   contact_name varchar(255) NOT NULL,
//   contact_phone varchar(16) NOT NULL,
//   contact_email varchar(50) NOT NULL,
//   start_time datetime NOT NULL,
//   location varchar(255) NOT NULL,
//   comments text DEFAULT NULL,
//   status enum('pending', 'accepted', 'dismissed') NOT NULL DEFAULT 'pending',

//   FOREIGN KEY (event) REFERENCES event (id) ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "gig_request"]
pub struct GigRequest {
    pub id: i32,
    pub time: NaiveDateTime,
    pub name: String,
    pub organization: String,
    pub event: Option<i32>,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub start_time: NaiveDateTime,
    pub location: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub comments: Option<String>,
    pub status: GigRequestStatus,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract, Insertable)]
#[table_name = "gig_request"]
pub struct NewGigRequest {
    pub name: String,
    pub organization: String,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub start_time: NaiveDateTime,
    pub location: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub comments: Option<String>,
}

#[derive(Deserialize, Extract)]
pub struct GigRequestForm {
    #[serde(flatten)]
    pub event: NewEvent,
    #[serde(flatten)]
    pub gig: NewGig,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum GigRequestStatus {
    #[strum(serialize = "pending")]
    Pending,
    #[strum(serialize = "accepted")]
    Accepted,
    #[strum(serialize = "dismissed")]
    Dismissed,
}

// CREATE TABLE song (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   title varchar(128) NOT NULL,
//   info text DEFAULT NULL,
//   current boolean NOT NULL DEFAULT '0',
//   `key` enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯', 'E♭',
//              'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
//   starting_pitch enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯',
//                       'E♭', 'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
//   mode enum('major', 'minor', 'dorian', 'phrygian', 'lydian',
//             'mixolydian', 'locrian') DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract)]
#[table_name = "song"]
pub struct Song {
    pub id: i32,
    pub title: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub info: Option<String>,
    pub current: bool,
    pub key: Option<Key>,
    pub starting_pitch: Option<Key>,
    pub mode: Option<SongMode>,
}

#[derive(TableName, Deserialize, Extract, Insertable)]
#[table_name = "song"]
pub struct NewSong {
    pub title: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum Key {
    #[serde(rename = "A♭")]
    #[strum(serialize = "A♭")]
    AFlat,
    #[serde(rename = "A")]
    #[strum(serialize = "A")]
    A,
    #[serde(rename = "A♯")]
    #[strum(serialize = "A♯")]
    ASharp,
    #[serde(rename = "B♭")]
    #[strum(serialize = "B♭")]
    BFlat,
    #[serde(rename = "B")]
    #[strum(serialize = "B")]
    B,
    #[serde(rename = "B♯")]
    #[strum(serialize = "B♯")]
    BSharp,
    #[serde(rename = "C♭")]
    #[strum(serialize = "C♭")]
    CFlat,
    #[serde(rename = "C")]
    #[strum(serialize = "C")]
    C,
    #[serde(rename = "C♯")]
    #[strum(serialize = "C♯")]
    CSharp,
    #[serde(rename = "D♭")]
    #[strum(serialize = "D♭")]
    DFlat,
    #[serde(rename = "D")]
    #[strum(serialize = "D")]
    D,
    #[serde(rename = "D♯")]
    #[strum(serialize = "D♯")]
    DSharp,
    #[serde(rename = "E♭")]
    #[strum(serialize = "E♭")]
    EFlat,
    #[serde(rename = "E")]
    #[strum(serialize = "E")]
    E,
    #[serde(rename = "E♯")]
    #[strum(serialize = "E♯")]
    ESharp,
    #[serde(rename = "F♭")]
    #[strum(serialize = "F♭")]
    FFlat,
    #[serde(rename = "F")]
    #[strum(serialize = "F")]
    F,
    #[serde(rename = "F♯")]
    #[strum(serialize = "F♯")]
    FSharp,
    #[serde(rename = "G♭")]
    #[strum(serialize = "G♭")]
    GFlat,
    #[serde(rename = "G")]
    #[strum(serialize = "G")]
    G,
    #[serde(rename = "G♯")]
    #[strum(serialize = "G♯")]
    GSharp,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum SongMode {
    #[strum(serialize = "major")]
    Major,
    #[strum(serialize = "minor")]
    Minor,
    #[strum(serialize = "dorian")]
    Dorian,
    #[strum(serialize = "phrygian")]
    Phrygian,
    #[strum(serialize = "lydian")]
    Lydian,
    #[strum(serialize = "myxolydian")]
    Myxolydian,
    #[strum(serialize = "locrian")]
    Locrian,
}

// CREATE TABLE gig_song (
//   event int NOT NULL,
//   song int NOT NULL,
//   `order` int NOT NULL,

//   PRIMARY KEY (event, song),
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable)]
#[table_name = "gig_song"]
pub struct GigSong {
    pub event: i32,
    pub song: i32,
    pub order: i32,
}

#[derive(Deserialize, Extract)]
pub struct NewGigSong {
    pub song: i32,
}

// CREATE TABLE media_type (
//   name varchar(50) NOT NULL PRIMARY KEY,
//   `order` int NOT NULL UNIQUE,
//   storage enum('local', 'remote') NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "media_type"]
pub struct MediaType {
    pub name: String,
    pub order: i32,
    pub storage: StorageType,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum StorageType {
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "remote")]
    Remote,
}

// CREATE TABLE minutes (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   name varchar(100) NOT NULL,
//   `date` date NOT NULL,
//   private longtext DEFAULT NULL,
//   public longtext DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "minutes"]
pub struct MeetingMinutes {
    pub id: i32,
    pub name: String,
    pub date: NaiveDate,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub private: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub public: Option<String>,
}

// CREATE TABLE permission (
//   name varchar(40) NOT NULL PRIMARY KEY,
//   description text DEFAULT NULL,
//   `type` enum('static', 'event') NOT NULL DEFAULT 'static'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "permission"]
pub struct Permission {
    pub name: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub description: Option<String>,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: PermissionType,
}

#[derive(Debug, Clone, PartialEq, MysqlEnum, Serialize, Deserialize, Display, EnumString)]
pub enum PermissionType {
    #[strum(serialize = "static")]
    Static,
    #[strum(serialize = "event")]
    Event,
}

// CREATE TABLE rides_in (
//   member varchar(50) NOT NULL,
//   carpool int NOT NULL,

//   PRIMARY KEY (member, carpool),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (carpool) REFERENCES carpool (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable)]
#[table_name = "rides_in"]
pub struct RidesIn {
    pub member: String,
    pub carpool: i32,
}

// CREATE TABLE role_permission (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   role varchar(20) NOT NULL,
//   permission varchar(40) NOT NULL,
//   event_type varchar(32) DEFAULT NULL,

//   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (permission) REFERENCES permission (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event_type) REFERENCES event_type (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "role_permission"]
pub struct RolePermission {
    pub id: i32,
    pub role: String,
    pub permission: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub event_type: Option<String>,
}

// CREATE TABLE song_link (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   song int NOT NULL,
//   `type` varchar(50) NOT NULL,
//   name varchar(128) NOT NULL,
//   target varchar(255) NOT NULL,

//   FOREIGN KEY (`type`) REFERENCES media_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "song_link"]
pub struct SongLink {
    pub id: i32,
    pub song: i32,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub target: String,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract)]
#[table_name = "song_link"]
pub struct NewSongLink {
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub target: String,
}

#[derive(Deserialize, Extract)]
pub struct SongLinkUpdate {
    pub name: String,
    pub target: String,
}

// CREATE TABLE todo (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   `text` varchar(255) NOT NULL,
//   member varchar(50) NOT NULL,
//   completed boolean NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON UPDATE CASCADE ON DELETE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "todo"]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub member: String,
    pub completed: bool,
}

#[derive(Deserialize, Extract)]
pub struct NewTodo {
    pub text: String,
    pub members: Vec<String>,
}

// CREATE TABLE transaction_type (
//   name varchar(40) NOT NULL PRIMARY KEY
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "transaction_type"]
pub struct TransactionType {
    pub name: String,
}

// CREATE TABLE transaction (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   member varchar(50) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   amount int NOT NULL,
//   description varchar(500) NOT NULL,
//   semester varchar(32) DEFAULT NULL,
//   `type` varchar(40) NOT NULL,
//   resolved tinyint(1) NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (`type`) REFERENCES transaction_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames)]
#[table_name = "transaction"]
pub struct Transaction {
    pub id: i32,
    pub member: String,
    pub time: NaiveDateTime,
    pub amount: i32,
    pub description: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub semester: Option<String>,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub resolved: bool,
}

#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Extract, Insertable)]
#[table_name = "transaction"]
pub struct NewTransaction {
    pub member: String,
    pub amount: i32,
    pub description: String,
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub semester: Option<String>,
    #[rename = "type"]
    #[serde(rename = "type")]
    pub type_: String,
    pub resolved: bool,
}

// CREATE TABLE session (
//   member varchar(50) NOT NULL PRIMARY KEY,
//   `key` varchar(64) NOT NULL,

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable, Debug)]
#[table_name = "session"]
pub struct Session {
    pub member: String,
    pub key: String,
}

// CREATE TABLE variable (
//   `key` varchar(255) NOT NULL PRIMARY KEY,
//   value varchar(255) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
#[derive(TableName, FromRow, Serialize, Deserialize, FieldNames, Insertable)]
#[table_name = "variable"]
pub struct Variable {
    pub key: String,
    pub value: String,
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|s| Some(s).filter(|s| s.len() > 0))
}
