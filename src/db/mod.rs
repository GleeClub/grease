//! Database interaction for the API.

pub mod models;
pub mod schema;

use self::schema::{
    absence_request, active_semester, announcement, attendance, carpool, event, event_type, fee,
    gig, gig_request, gig_song, google_docs, media_type, member, member_role, minutes, permission,
    rides_in, role, role_permission, semester, session, song, song_link, todo, transaction,
    transaction_type, uniform, variable, AbsenceRequestState, Enrollment, GigRequestStatus,
    PermissionType, Pitch, SongMode, StorageType,
};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{Connection, MysqlConnection};
use diesel::{associations::Identifiable, AsChangeset, Insertable, Queryable};
use error::{GreaseError, GreaseResult};
use serde::{de::Error as _, de::Unexpected, Deserialize, Deserializer, Serialize};

pub fn connect_to_db() -> GreaseResult<MysqlConnection> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_err| GreaseError::ServerError("Database url missing".to_owned()))?;

    MysqlConnection::establish(&db_url).map_err(GreaseError::ConnectionError)
}

/// The model for members.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE member (
///   email varchar(50) NOT NULL PRIMARY KEY,
///   first_name varchar(25) NOT NULL,
///   preferred_name varchar(25) DEFAULT NULL,
///   last_name varchar(25) NOT NULL,
///   pass_hash varchar(64) NOT NULL,
///   phone_number varchar(16) NOT NULL,
///   picture varchar(255) DEFAULT NULL,
///   passengers int NOT NULL DEFAULT '0',
///   location varchar(50) NOT NULL,
///   on_campus tinyint(1) DEFAULT NULL,
///   about varchar(500) DEFAULT NULL,
///   major varchar(50) DEFAULT NULL,
///   minor varchar(50) DEFAULT NULL,
///   hometown varchar(50) DEFAULT NULL,
///   arrived_at_tech int DEFAULT NULL, -- year
///   gateway_drug varchar(500) DEFAULT NULL,
///   conflicts varchar(500) DEFAULT NULL,
///   dietary_restrictions varchar(500) DEFAULT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "email": string,
///     "firstName": string,
///     "preferredName": string?,
///     "lastName": string,
///     "fullName": string,
///     "phoneNumber": string,
///     "picture": string?,
///     "passengers": integer,
///     "location": string,
///     "onCampus": boolean?,
///     "about": string?,
///     "major": string?,
///     "minor": string?,
///     "hometown": string?,
///     "arrivedAtTech": integer?,
///     "gatewayDrug": string?,
///     "conflicts": string?,
///     "dietaryRestrictions": string?
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Clone)]
#[table_name = "member"]
#[primary_key(email)]
pub struct Member {
    /// The member's email, which must be unique
    pub email: String,
    /// The member's first name
    #[serde(rename = "firstName")]
    /// The member's nick name
    pub first_name: String,
    #[serde(rename = "preferredName", deserialize_with = "deser_opt_string")]
    pub preferred_name: Option<String>,
    /// The member's last name
    #[serde(rename = "lastName")]
    pub last_name: String,
    /// The hash of the member's password
    #[serde(rename = "passHash", skip_serializing)]
    pub pass_hash: String,
    /// The member's phone number
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    /// An optional link to a profile picture for the member
    #[serde(deserialize_with = "deser_opt_string")]
    pub picture: Option<String>,
    /// The number of people the member is able to drive for gigs
    pub passengers: i32,
    /// Where the member lives
    pub location: String,
    /// Whether the member currently lives on campus (assumed false)
    #[serde(default, rename = "onCampus")]
    pub on_campus: Option<bool>,
    /// An optional bio for the member
    #[serde(deserialize_with = "deser_opt_string")]
    pub about: Option<String>,
    /// The member's academic major
    #[serde(deserialize_with = "deser_opt_string")]
    pub major: Option<String>,
    /// The member's academic minor
    #[serde(deserialize_with = "deser_opt_string")]
    pub minor: Option<String>,
    /// Where the member originally comes from
    #[serde(deserialize_with = "deser_opt_string")]
    pub hometown: Option<String>,
    /// What year the member arrived at Tech (e.g. 2012)
    #[serde(rename = "arrivedAtTech")]
    pub arrived_at_tech: Option<i32>,
    /// What brought the member to Glee Club
    #[serde(rename = "gatewayDrug", deserialize_with = "deser_opt_string")]
    pub gateway_drug: Option<String>,
    /// What conflicts during the week the member may have
    #[serde(deserialize_with = "deser_opt_string")]
    pub conflicts: Option<String>,
    /// What dietary restrictions the member may have
    #[serde(rename = "dietaryRestrictions", deserialize_with = "deser_opt_string")]
    pub dietary_restrictions: Option<String>,
}

/// The required format for adding new members and updating existing ones.
///
/// ## Expected Format for New Members:
///
/// |        Field        |     Type     | Required? | Comments |
/// |---------------------|--------------|:---------:|----------|
/// | email               | string       |     ✓     |          |
/// | firstName           | string       |     ✓     |          |
/// | preferredName       | string       |           |          |
/// | lastName            | string       |     ✓     |          |
/// | passHash            | string       |     ✓     |          |
/// | phoneNumber         | string       |     ✓     |          |
/// | passengers          | integer      |     ✓     |          |
/// | location            | string       |     ✓     |          |
/// | onCampus            | boolean      |           |          |
/// | about               | string       |           |          |
/// | major               | string       |           |          |
/// | minor               | string       |           |          |
/// | hometown            | string       |           |          |
/// | arrivedAtTech       | integer      |           |          |
/// | gatewayDrug         | string       |           |          |
/// | conflicts           | string       |           |          |
/// | dietaryRestrictions | string       |           |          |
/// | enrollment          | [Enrollment] |     ✓     |          |
/// | section             | string       |     ✓     |          |
///
/// ## Expected Format for Member Updates:
///
/// |        Field        |     Type     | Required? | Comments |
/// |---------------------|--------------|:---------:|----------|
/// | email               | string       |     ✓     |          |
/// | firstName           | string       |     ✓     |          |
/// | preferredName       | string       |           |          |
/// | lastName            | string       |     ✓     |          |
/// | passHash            | string       |           | officers can't change members' passwords |
/// | phoneNumber         | string       |     ✓     |          |
/// | passengers          | integer      |     ✓     |          |
/// | location            | string       |     ✓     |          |
/// | onCampus            | boolean      |           |          |
/// | about               | string       |           |          |
/// | major               | string       |           |          |
/// | minor               | string       |           |          |
/// | hometown            | string       |           |          |
/// | arrivedAtTech       | integer      |           |          |
/// | gatewayDrug         | string       |           |          |
/// | conflicts           | string       |           |          |
/// | dietaryRestrictions | string       |           |          |
/// | enrollment          | [Enrollment] |     ✓     |          |
/// | section             | string       |     ✓     |          |
///
/// [Enrollment]: enum.Enrollment.html
#[derive(Deserialize)]
pub struct NewMember {
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(
        default,
        rename = "preferredName",
        deserialize_with = "deser_opt_string"
    )]
    pub preferred_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(default, rename = "passHash", deserialize_with = "deser_opt_string")]
    pub pass_hash: Option<String>,
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    #[serde(default, rename = "onCampus")]
    pub on_campus: Option<bool>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub about: Option<String>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub major: Option<String>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub minor: Option<String>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub hometown: Option<String>,
    #[serde(default, rename = "arrivedAtTech")]
    pub arrived_at_tech: Option<i32>,
    #[serde(default, rename = "gatewayDrug", deserialize_with = "deser_opt_string")]
    pub gateway_drug: Option<String>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub conflicts: Option<String>,
    #[serde(
        default,
        rename = "dietaryRestrictions",
        deserialize_with = "deser_opt_string"
    )]
    pub dietary_restrictions: Option<String>,
    #[serde(default)]
    pub enrollment: Option<Enrollment>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub section: Option<String>,
}

/// The required format when members confirm activity for a semester.
///
/// ## Expected Format:
///
/// |        Field        |     Type     | Required? | Comments |
/// |---------------------|--------------|:---------:|----------|
/// | location            | string       |     ✓     |          |
/// | onCampus            | boolean      |           |          |
/// | conflicts           | string       |           |          |
/// | dietaryRestrictions | string       |           |          |
/// | enrollment          | [Enrollment] |     ✓     |          |
/// | section             | string       |     ✓     |          |
///
/// [Enrollment]: enum.Enrollment.html
#[derive(Deserialize)]
pub struct RegisterForSemesterForm {
    pub location: String,
    #[serde(default, rename = "onCampus")]
    pub on_campus: Option<bool>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub conflicts: Option<String>,
    #[serde(rename = "dietaryRestrictions", deserialize_with = "deser_opt_string")]
    pub dietary_restrictions: Option<String>,
    pub enrollment: Enrollment,
    pub section: String,
}

/// The model for semesters.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE semester (
///   name varchar(32) NOT NULL PRIMARY KEY,
///   start_date datetime NOT NULL,
///   end_date datetime NOT NULL,
///   gig_requirement int NOT NULL DEFAULT '5',
///   current boolean NOT NULL DEFAULT '0'
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "startDate": datetime,
///     "endDate": datetime,
///     "gigRequirement": boolean,
///     "current": boolean
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "semester"]
#[primary_key(name)]
pub struct Semester {
    /// The name of the semester
    pub name: String,
    /// When the semester starts
    #[serde(rename = "startDate", with = "naivedatetime_posix")]
    pub start_date: NaiveDateTime,
    /// When the semester ends
    #[serde(rename = "endDate", with = "naivedatetime_posix")]
    pub end_date: NaiveDateTime,
    /// How many volunteer gigs are required for the semester
    #[serde(rename = "gigRequirement")]
    pub gig_requirement: i32,
    /// Whether this is the current semester
    pub current: bool,
}

/// The required format for creating and updating semesters.
///
/// ## Expected Format:
///
/// |     Field      |   Type   | Required? |         Comments          |
/// |----------------|----------|:---------:|---------------------------|
/// | name           | string   |     ✓     |                           |
/// | startDate      | datetime |     ✓     |                           |
/// | endDate        | datetime |     ✓     | must be after `startDate` |
/// | gigRequirement | integer  |     ✓     |                           |
#[derive(Deserialize, Insertable)]
#[table_name = "semester"]
pub struct NewSemester {
    pub name: String,
    #[serde(rename = "startDate", with = "naivedatetime_posix")]
    pub start_date: NaiveDateTime,
    #[serde(rename = "endDate", with = "naivedatetime_posix")]
    pub end_date: NaiveDateTime,
    #[serde(rename = "gigRequirement")]
    pub gig_requirement: i32,
}

/// The model for roles within the organization (a.k.a. officer positions).
///
/// ## Database Format:
///
/// ```sql
///  CREATE TABLE role (
///   name varchar(20) NOT NULL PRIMARY KEY,
///   `rank` int NOT NULL,
///   max_quantity int NOT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "rank": integer,
///     "maxQuantity": integer
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "role"]
#[primary_key(name)]
pub struct Role {
    /// The name of the role
    pub name: String,
    /// Used for ordering the positions (e.g. President before Ombudsman)
    pub rank: i32,
    /// The maximum number of the position allowed to be held at once.
    /// If it is 0 or less, no maximum is enforced.
    #[serde(rename = "maxQuantity")]
    pub max_quantity: i32,
}

/// The model for the recording which member holds what role.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE member_role (
///   member varchar(50) NOT NULL,
///   role varchar(20) NOT NULL,
///
///   PRIMARY KEY (member, role),
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "member": string,
///     "role": string
/// }
/// ```
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "member_role"]
pub struct MemberRole {
    /// The email of the member holding the role
    pub member: String,
    /// The name of the role being held
    pub role: String,
}

/// The names of the sections that members sing in (e.g. Baritone).
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE section_type (
///   name varchar(20) NOT NULL PRIMARY KEY
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string
/// }
/// ```
#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct SectionType {
    /// The name of the section type
    pub name: String,
}

/// The types of events that members will attend.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE event_type (
///   name varchar(32) NOT NULL PRIMARY KEY,
///   weight int NOT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "weight": integer
/// }
/// ```
#[derive(Queryable, Serialize, Deserialize, PartialEq)]
pub struct EventType {
    /// The name of the type of event
    pub name: String,
    /// How many points this type is worth
    pub weight: i32,
}

/// The model for events that members attend.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE event (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   name varchar(64) NOT NULL,
///   semester varchar(32) NOT NULL,
///   `type` varchar(32) NOT NULL,
///   call_time datetime NOT NULL,
///   release_time datetime DEFAULT NULL,
///   points int NOT NULL,
///   comments text DEFAULT NULL,
///   location varchar(255) DEFAULT NULL,
///   gig_count boolean NOT NULL DEFAULT '1',
///   default_attend boolean NOT NULL DEFAULT '1',
///   section varchar(20) DEFAULT NULL,
///
///   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE ON DELETE CASCADE,
///   FOREIGN KEY (`type`) REFERENCES event_type (name) ON UPDATE CASCADE ON DELETE CASCADE,
///   FOREIGN KEY (section) REFERENCES section_type (name) ON UPDATE CASCADE ON DELETE SET NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// `Event`s are not directly serialized, see [to_json](event/struct.EventWithGig.html#method.to_json)
/// for how `Event`s get serialized with `Gig`s.
#[derive(Queryable, Serialize, Deserialize, Clone)]
pub struct Event {
    /// The ID of the event
    pub id: i32,
    /// The name of the event
    pub name: String,
    /// The name of the [Semester](struct.Semester.html) this event belongs to
    pub semester: String,
    /// The type of the event (see [EventType](struct.EventType.html))
    #[serde(rename = "type")]
    pub type_: String,
    /// When members are expected to arrive to the event
    #[serde(rename = "callTime", with = "naivedatetime_posix")]
    pub call_time: NaiveDateTime,
    /// When members are probably going to be released
    #[serde(rename = "releaseTime", with = "optional_naivedatetime_posix")]
    pub release_time: Option<NaiveDateTime>,
    /// How many points attendance of this event is worth
    pub points: i32,
    /// General information or details about this event
    #[serde(deserialize_with = "deser_opt_string")]
    pub comments: Option<String>,
    /// Where this event will be held
    #[serde(deserialize_with = "deser_opt_string")]
    pub location: Option<String>,
    /// Whether this event counts toward the volunteer gig count for the semester
    #[serde(rename = "gigCount")]
    pub gig_count: bool,
    /// Whether members are assumed to attend (most events)
    #[serde(rename = "defaultAttend")]
    pub default_attend: bool,
    /// If this event is for one singing [section](struct.SectionType.html) only,
    /// this denotes which one (e.g. old sectionals)
    #[serde(deserialize_with = "deser_opt_string")]
    pub section: Option<String>,
}

/// The required format for adding events.
///
/// ## Expected Format:
///
/// |     Field     |   Type   | Required? |           Comments            |
/// |---------------|----------|:---------:|-------------------------------|
/// | name          | string   |     ✓     |                               |
/// | semester      | string   |     ✓     |                               |
/// | type          | string   |     ✓     | event type                    |
/// | callTime      | datetime |     ✓     |                               |
/// | releaseTime   | datetime |           |                               |
/// | points        | integer  |     ✓     |                               |
/// | comments      | string   |           |                               |
/// | location      | string   |           |                               |
/// | gigCount      | boolean  |     ✓     | for volunteer gigs            |
/// | defaultAttend | boolean  |     ✓     | assume members should go      |
/// | repeat        | string   |     ✓     | see [Period](event/enum.Period.html) |
/// | repeatUntil   | datetime |           | needed if `repeat` isn't "no" |
#[derive(Deserialize)]
pub struct NewEvent {
    pub name: String,
    pub semester: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "callTime", with = "naivedatetime_posix")]
    pub call_time: NaiveDateTime,
    #[serde(rename = "releaseTime", with = "optional_naivedatetime_posix")]
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    #[serde(deserialize_with = "deser_opt_string")]
    pub comments: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub location: Option<String>,
    #[serde(default, rename = "gigCount")]
    pub gig_count: Option<bool>,
    #[serde(rename = "defaultAttend")]
    pub default_attend: bool,
    pub repeat: String,
    #[serde(rename = "repeatUntil", with = "optional_naivedate_posix")]
    pub repeat_until: Option<NaiveDate>,
}

/// The required format for updating events.
///
/// ## Expected Format:
///
/// |      Field       |      Type      |      Required?       |       Comments        |
/// |------------------|----------------|:--------------------:|-----------------------|
/// | name             | string         |          ✓           |                       |
/// | semester         | string         |          ✓           |                       |
/// | type             | string         |          ✓           | the event type        |
/// | callTime         | datetime       |          ✓           |                       |
/// | releaseTime      | datetime       |                      |                       |
/// | points           | integer        |          ✓           |                       |
/// | comments         | string         |                      |                       |
/// | location         | string         |                      |                       |
/// | gigCount         | boolean        |          ✓           | for volunteer gigs    |
/// | defaultAttend    | boolean        |          ✓           |                       |
/// | section          | string         |                      | name of the section   |
/// | performanceTime  | datetime       | for events with gigs |                       |
/// | uniform          | integer        | for events with gigs |                       |
/// | contactName      | string         |                      |                       |
/// | contactEmail     | string         |                      |                       |
/// | contactPhone     | string         |                      |                       |
/// | price            | integer        |                      |                       |
/// | public           | boolean        | for events with gigs | show on external site |
/// | summary          | string         |                      | public event summary  |
/// | description      | string         |                      | public event summary  |
#[derive(Deserialize, Debug)]
pub struct EventUpdate {
    // event fields
    pub name: String,
    pub semester: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "callTime", with = "naivedatetime_posix")]
    pub call_time: NaiveDateTime,
    #[serde(rename = "releaseTime", with = "optional_naivedatetime_posix")]
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    #[serde(deserialize_with = "deser_opt_string")]
    pub comments: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub location: Option<String>,
    #[serde(rename = "gigCount")]
    pub gig_count: bool,
    #[serde(rename = "defaultAttend")]
    pub default_attend: bool,
    #[serde(deserialize_with = "deser_opt_string")]
    pub section: Option<String>,
    // gig fields
    #[serde(rename = "performanceTime", with = "optional_naivedatetime_posix")]
    pub performance_time: Option<NaiveDateTime>,
    pub uniform: Option<i32>,
    #[serde(rename = "contactName", deserialize_with = "deser_opt_string")]
    pub contact_name: Option<String>,
    #[serde(rename = "contactEmail", deserialize_with = "deser_opt_string")]
    pub contact_email: Option<String>,
    #[serde(rename = "contactPhone", deserialize_with = "deser_opt_string")]
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: Option<bool>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub description: Option<String>,
}

/// The model for member's requests for absence from events.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE absence_request (
///   member varchar(50) NOT NULL,
///   event int NOT NULL,
///   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
///   reason varchar(500) NOT NULL,
///   state enum('pending', 'approved', 'denied') NOT NULL DEFAULT 'pending',
///
///   PRIMARY KEY (member, event),
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "member": string,
///     "event": integer,
///     "time": datetime,
///     "reason": string,
///     "state": string
/// }
/// ```
#[derive(Serialize, Clone)]
pub struct AbsenceRequest {
    /// The email of the member that requested an absence
    pub member: String,
    /// The ID of the event they requested absence from
    pub event: i32,
    /// The time this request was placed
    #[serde(with = "naivedatetime_posix")]
    pub time: NaiveDateTime,
    /// The reason the member petitioned for absence with
    pub reason: String,
    /// The current state of the request (See [AbsenceRequestState](enum.AbsenceRequestState.html))
    pub state: AbsenceRequestState,
}

/// The required format for new absence requests.
///
/// | Field  |  Type  | Required? | Comments |
/// |--------|--------|:---------:|----------|
/// | reason | string |     ✓     |          |
#[derive(Deserialize, Insertable)]
#[table_name = "absence_request"]
pub struct NewAbsenceRequest {
    pub reason: String,
}

/// The model that records which semesters a member has been active during.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE active_semester (
///   member varchar(50) NOT NULL,
///   semester varchar(32) NOT NULL,
///   enrollment enum('class', 'club') NOT NULL DEFAULT 'club',
///   section varchar(20) DEFAULT NULL,
///
///   PRIMARY KEY (member, semester),
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (section) REFERENCES section_type (name) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "member": string,
///     "semester": string,
///     "enrollment": string,
///     "section": string?
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize, Insertable, PartialEq, Debug)]
#[table_name = "active_semester"]
#[primary_key(member, semester)]
pub struct ActiveSemester {
    /// The email of the active member
    pub member: String,
    /// The name of the semester they were active during
    pub semester: String,
    /// Whether they were in the class or the club (See [Enrollment](enum.Enrollment.html))
    pub enrollment: Enrollment,
    /// Which section the member sang in (See [SectionType](struct.SectionType.html))
    #[serde(deserialize_with = "deser_opt_string")]
    pub section: Option<String>,
}

/// The required format for updating active semesters for members.
///
/// ## Expected Format:
///
/// |   Field    |  Type  | Required? | Comments |
/// |------------|--------|:---------:|----------|
/// | enrollment | string |     ✓     |          |
/// | section    | string |           |          |
#[derive(Deserialize, AsChangeset)]
#[table_name = "active_semester"]
pub struct ActiveSemesterUpdate {
    #[serde(deserialize_with = "deser_enrollment")]
    pub enrollment: Option<Enrollment>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub section: Option<String>,
}

/// The model for announcements made to the club.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE announcement (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   member varchar(50) DEFAULT NULL,
///   semester varchar(32) NOT NULL,
///   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
///   content longtext NOT NULL,
///   archived bool NOT NULL DEFAULT '0',
///
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE SET NULL ON UPDATE CASCADE,
///   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "member": string, // null if deleted
///     "semester": string,
///     "time": datetime,
///     "content": string,
///     "archived": boolean
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "announcement"]
pub struct Announcement {
    /// The ID of the announcement
    pub id: i32,
    /// The email of the member who made the announcement (null if the member was later deleted)
    #[serde(deserialize_with = "deser_opt_string")]
    pub member: Option<String>,
    /// The name of the semester the announcement was made during
    pub semester: String,
    /// When the announcement was made
    #[serde(with = "naivedatetime_posix")]
    pub time: NaiveDateTime,
    /// The content of the announcement
    pub content: String,
    /// Whether an officer archived the announcement
    pub archived: bool,
}

/// The required format for making new announcements.
///
/// ## Expected Format:
///
/// |  Field  |  Type  | Required? | Comments |
/// |---------|--------|:---------:|----------|
/// | content | string |     ✓     |          |
#[derive(Deserialize)]
pub struct NewAnnouncement {
    pub content: String,
}

/// The model for member attendance.
///
/// ## Database Format:
/// ```sql
/// CREATE TABLE attendance (
///   member varchar(50) NOT NULL,
///   event int NOT NULL,
///   should_attend boolean NOT NULL DEFAULT '1',
///   did_attend boolean NOT NULL DEFAULT '0',
///   confirmed boolean NOT NULL DEFAULT '0',
///   minutes_late int NOT NULL DEFAULT '0',
///
///   PRIMARY KEY (member, event),
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
/// ```json
/// {
///     "member": string,
///     "event": integer,
///     "shouldAttend": boolean,
///     "didAttend": boolean,
///     "confirmed": boolean,
///     "minutesLate": integer
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Clone)]
#[table_name = "attendance"]
#[primary_key(member, event)]
pub struct Attendance {
    /// The email of the member this attendance belongs to
    pub member: String,
    /// The ID of the event this attendance belongs to
    pub event: i32,
    /// Whether the member is expected to attend the event
    #[serde(rename = "shouldAttend")]
    pub should_attend: bool,
    /// Whether the member did attend the event
    #[serde(rename = "didAttend")]
    pub did_attend: bool,
    /// Whether the member confirmed that they would attend
    pub confirmed: bool,
    /// How late the member was if they attended
    #[serde(rename = "minutesLate")]
    pub minutes_late: i32,
}

/// The required format for updating a member's attendance.
///
/// ## Expected Format:
///
/// |    Field     |  Type   | Required? | Comments |
/// |--------------|---------|:---------:|----------|
/// | shouldAttend | boolean |     ✓     |          |
/// | didAttend    | boolean |     ✓     |          |
/// | minutesLate  | integer |     ✓     |          |
/// | confirmed    | boolean |     ✓     |          |
#[derive(Deserialize)]
pub struct AttendanceForm {
    #[serde(rename = "shouldAttend")]
    pub should_attend: bool,
    #[serde(rename = "didAttend")]
    pub did_attend: bool,
    #[serde(rename = "minutesLate")]
    pub minutes_late: i32,
    pub confirmed: bool,
}

#[derive(Insertable, Deserialize)]
#[table_name = "attendance"]
pub struct NewAttendance {
    pub event: i32,
    #[serde(rename = "shouldAttend")]
    pub should_attend: bool,
    pub member: String,
}

/// The model for recording who is driving who for events.
///
/// ## Datbase Format:
///
/// ```sql
/// CREATE TABLE carpool (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   event int NOT NULL,
///   driver varchar(50) NOT NULL,
///
///   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (driver) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "event": integer,
///     "driver": string
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "carpool"]
pub struct Carpool {
    /// The ID of the carpool
    pub id: i32,
    /// The ID of the event this carpool belongs to
    pub event: i32,
    /// The email of the driver of this carpool
    pub driver: String,
}

/// The required format for making new carpools.
///
/// ## Expected Format:
///
/// | Field  |  Type   | Required? | Comments |
/// |--------|---------|:---------:|----------|
/// | event  | integer |     ✓     |          |
/// | driver | string  |     ✓     |          |
#[derive(Deserialize, Insertable)]
#[table_name = "carpool"]
pub struct NewCarpool {
    pub event: i32,
    pub driver: String,
}

/// The required format for updating the carpools for an event.
///
/// ## Expected Format:
///
/// |    Field   |    Type    | Required? |                 Comments                  |
/// |------------|------------|:---------:|-------------------------------------------|
/// | driver     | string     |     ✓     | the email of the driver                   |
/// | passengers | \[string\] |     ✓     | the emails of the passengers              |
#[derive(Deserialize)]
pub struct UpdatedCarpool {
    pub driver: String,
    pub passengers: Vec<String>,
}

/// The model for fees to charge members.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE fee (
///   name varchar(16) NOT NULL PRIMARY KEY,
///   description varchar(40) NOT NULL,
///   amount int NOT NULL DEFAULT '0'
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "description": string,
///     "amount": integer
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "fee"]
#[primary_key(name)]
pub struct Fee {
    /// The short name of the fee
    pub name: String,
    /// A longer description of what it is charging members for
    pub description: String,
    /// The amount to charge members
    pub amount: i32,
}

/// The model for Google Docs.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE google_docs (
///   name varchar(40) NOT NULL PRIMARY KEY,
///   url varchar(255) NOT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "url": string
/// }
/// ```
///
/// ## Expected Format for New Google Docs:
///
/// | Field |  Type  | Required? | Comments |
/// |-------|--------|:---------:|----------|
/// | name  | string |     ✓     |          |
/// | url   | string |     ✓     |          |
#[derive(Identifiable, Insertable, Serialize, Deserialize)]
#[table_name = "google_docs"]
#[primary_key(name)]
pub struct GoogleDoc {
    /// The name of the Google Doc
    pub name: String,
    /// A link to the Google Doc (must be an embed link)
    pub url: String,
}

/// The model for uniforms that members wear to events.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE uniform (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   name varchar(32) NOT NULL,
///   color varchar(4) DEFAULT NULL,
///   description text DEFAULT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "name": string,
///     "color": string?,
///     "description": string?
/// }
/// ```
#[derive(Identifiable, Serialize)]
#[table_name = "uniform"]
pub struct Uniform {
    /// The ID of the uniform
    pub id: i32,
    /// The name of the uniform
    pub name: String,
    /// The associated color of the uniform (In the format "#XXX", where "X" is a hex digit)
    #[serde(deserialize_with = "deser_opt_string")]
    pub color: Option<String>,
    /// The explanation of what to wear when wearing the uniform
    #[serde(deserialize_with = "deser_opt_string")]
    pub description: Option<String>,
}

/// The required format for new uniforms and uniform updates.
///
/// ## Expected Format:
///
/// |    Field    |  Type  | Required? |                   Comments                    |
/// |-------------|--------|:---------:|-----------------------------------------------|
/// | name        | string |     ✓     |                                               |
/// | color       | string |           | must be formatted "#XXX", X being a hex digit |
/// | description | string |           |                                               |
#[derive(Insertable, Deserialize)]
#[table_name = "uniform"]
pub struct NewUniform {
    pub name: String,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub color: Option<String>,
    #[serde(default, deserialize_with = "deser_opt_string")]
    pub description: Option<String>,
}

/// The models for gigs tied to events.
///
/// ### Database Format:
///
/// ```sql
/// CREATE TABLE gig (
///   event int NOT NULL PRIMARY KEY,
///   performance_time datetime NOT NULL,
///   uniform int NOT NULL,
///   contact_name varchar(50) DEFAULT NULL,
///   contact_email varchar(50) DEFAULT NULL,
///   contact_phone varchar(16) DEFAULT NULL,
///   price int DEFAULT NULL,
///   public boolean NOT NULL DEFAULT '0',
///   summary text DEFAULT NULL,
///   description text DEFAULT NULL,
///
///   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (uniform) REFERENCES uniform (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// `Gig`s are not directly serialized, see [to_json](event/struct.EventWithGig.html#method.to_json)
/// for how `Gig`s get serialized with `Event`s.
#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "gig"]
#[primary_key(event)]
pub struct Gig {
    /// The ID of the event this gig belongs to
    pub event: i32,
    /// When members are expected to actually perform
    #[serde(rename = "performanceTime", with = "naivedatetime_posix")]
    pub performance_time: NaiveDateTime,
    /// The ID of the uniform for this gig
    pub uniform: i32,
    /// The name of the contact for this gig
    #[serde(rename = "contactName", deserialize_with = "deser_opt_string")]
    pub contact_name: Option<String>,
    /// The email of the contact for this gig
    #[serde(rename = "contactEmail", deserialize_with = "deser_opt_string")]
    pub contact_email: Option<String>,
    /// The phone of the contact for this gig
    #[serde(rename = "contactPhone", deserialize_with = "deser_opt_string")]
    pub contact_phone: Option<String>,
    /// The price we are charging for this gig
    pub price: Option<i32>,
    /// Whether this gig is visible on the external website
    pub public: bool,
    #[serde(deserialize_with = "deser_opt_string")]
    /// A summary of this event for the external site (if it is public)
    pub summary: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    /// A description of this event for the external site (if it is public)
    pub description: Option<String>,
}

/// The required format for the creation of new gigs.
///
/// ## Expected Format:
///
/// |     Field       |   Type   | Required? | Comments |
/// |-----------------|----------|:---------:|----------|
/// | performanceTime | datetime |     ✓     |          |
/// | uniform         | integer  |     ✓     |          |
/// | contactName     | string   |           |          |
/// | contactEmail    | string   |           |          |
/// | contactPhone    | string   |           |          |
/// | price           | integer  |           |          |
/// | public          | boolean  |     ✓     |          |
/// | summary         | string   |           |          |
/// | description     | string   |           |          |
#[derive(Insertable, Deserialize)]
#[table_name = "gig"]
pub struct NewGig {
    #[serde(rename = "performanceTime", with = "naivedatetime_posix")]
    pub performance_time: NaiveDateTime,
    pub uniform: i32,
    #[serde(rename = "contactName", deserialize_with = "deser_opt_string")]
    pub contact_name: Option<String>,
    #[serde(rename = "contactEmail", deserialize_with = "deser_opt_string")]
    pub contact_email: Option<String>,
    #[serde(rename = "contactPhone", deserialize_with = "deser_opt_string")]
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    #[serde(deserialize_with = "deser_opt_string")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub description: Option<String>,
}

/// The model for requests for Glee Club to perform.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE gig_request (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
///   name varchar(255) NOT NULL,
///   organization varchar(255) NOT NULL,
///   event int DEFAULT NULL,
///   contact_name varchar(255) NOT NULL,
///   contact_phone varchar(16) NOT NULL,
///   contact_email varchar(50) NOT NULL,
///   start_time datetime NOT NULL,
///   location varchar(255) NOT NULL,
///   comments text DEFAULT NULL,
///   status enum('pending', 'accepted', 'dismissed') NOT NULL DEFAULT 'pending',
///
///   FOREIGN KEY (event) REFERENCES event (id) ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "time": datetime,
///     "name": string,
///     "organization": string,
///     "event": integer?,
///     "contactName": string,
///     "contactEmail": string,
///     "contactPhone": string,
///     "startTime": datetime,
///     "location": string,
///     "comments": string?,
///     "status": string
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "gig_request"]
pub struct GigRequest {
    /// The ID of the gig request
    pub id: i32,
    /// When the gig request was placed
    #[serde(with = "naivedatetime_posix")]
    pub time: NaiveDateTime,
    /// The name of the potential event
    pub name: String,
    /// The organization requesting a performance from the Glee Club
    pub organization: String,
    /// If and when an event is created from a request, this is the event's ID
    pub event: Option<i32>,
    /// The name of the contact for the potential event
    #[serde(rename = "contactName")]
    pub contact_name: String,
    /// The email of the contact for the potential event
    #[serde(rename = "contactEmail")]
    pub contact_email: String,
    /// The phone number of the contact for the potential event
    #[serde(rename = "contactPhone")]
    pub contact_phone: String,
    /// When the event will probably happen
    #[serde(rename = "startTime", with = "naivedatetime_posix")]
    pub start_time: NaiveDateTime,
    /// Where the event will be happening
    pub location: String,
    /// Any comments about the event
    #[serde(deserialize_with = "deser_opt_string")]
    pub comments: Option<String>,
    /// The current status of whether the request was accepted
    pub status: GigRequestStatus,
}

/// The required format for the creation of new gigs.
///
/// ## Expected Format:
///
/// |    Field     |   Type   | Required? | Comments |
/// |--------------|----------|:---------:|----------|
/// | name         | string   |     ✓     |          |
/// | organization | string   |     ✓     |          |
/// | contactName  | string   |     ✓     |          |
/// | contactEmail | string   |     ✓     |          |
/// | contactPhone | string   |     ✓     |          |
/// | startTime    | datetime |     ✓     |          |
/// | location     | string   |     ✓     |          |
/// | comments     | string   |           |          |
#[derive(Deserialize, Insertable)]
#[table_name = "gig_request"]
pub struct NewGigRequest {
    pub name: String,
    pub organization: String,
    #[serde(rename = "contactName")]
    pub contact_name: String,
    #[serde(rename = "contactEmail")]
    pub contact_email: String,
    #[serde(rename = "contactPhone")]
    pub contact_phone: String,
    #[serde(rename = "startTime", with = "naivedatetime_posix")]
    pub start_time: NaiveDateTime,
    pub location: String,
    #[serde(deserialize_with = "deser_opt_string")]
    pub comments: Option<String>,
}

/// The required format for the creation of new events with gigs
/// from gig requests.
///
/// ## Expected Format:
///
/// |      Field      |   Type   | Required? |           Comments            |
/// |-----------------|----------|:---------:|-------------------------------|
/// | name            | string   |     ✓     |                               |
/// | semester        | string   |     ✓     |                               |
/// | type            | string   |     ✓     | event type                    |
/// | callTime        | datetime |     ✓     |                               |
/// | releaseTime     | datetime |           |                               |
/// | points          | integer  |     ✓     |                               |
/// | comments        | string   |           |                               |
/// | location        | string   |           |                               |
/// | defaultAttend   | boolean  |     ✓     | assume members should go      |
/// | repeat          | string   |     ✓     | see [Period](event/enum.Period.html) |
/// | repeatUntil     | datetime |           | needed if `repeat` isn't "no" |
/// | performanceTime | datetime |     ✓     |                               |
/// | uniform         | integer  |     ✓     |                               |
/// | contactName     | string   |           |                               |
/// | contactEmail    | string   |           |                               |
/// | contactPhone    | string   |           |                               |
/// | price           | integer  |           |                               |
/// | public          | boolean  |     ✓     |                               |
/// | summary         | string   |           |                               |
/// | description     | string   |           |                               |
#[derive(Deserialize)]
pub struct GigRequestForm {
    #[serde(flatten)]
    pub event: NewEvent,
    #[serde(flatten)]
    pub gig: NewGig,
}

/// The model for songs from our repertoire.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE song (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   title varchar(128) NOT NULL,
///   info text DEFAULT NULL,
///   current boolean NOT NULL DEFAULT '0',
///   `key` enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯', 'E♭',
///              'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
///   starting_pitch enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯',
///                       'E♭', 'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
///   mode enum('major', 'minor') DEFAULT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "title": string,
///     "info": string?,
///     "current": boolean,
///     "key": string?,
///     "startingPitch": string?,
///     "mode": string?
/// }
/// ```
#[derive(Queryable, Identifiable, Serialize)]
#[table_name = "song"]
pub struct Song {
    /// The ID of the song
    pub id: i32,
    /// The title of the song
    pub title: String,
    /// Any information related to the song (minor changes to the music, who wrote it, soloists, etc.)
    #[serde(deserialize_with = "deser_opt_string")]
    pub info: Option<String>,
    /// Whether it is in this semester's repertoire
    pub current: bool,
    /// The key of the song
    pub key: Option<Pitch>,
    /// The starting pitch for the song
    #[serde(rename = "startingPitch")]
    pub starting_pitch: Option<Pitch>,
    /// The mode of the song (Major or Minor)
    pub mode: Option<SongMode>,
}

/// The required format for the creation of new songs.
///
/// ## Expected Format:
///
/// | Field |  Type  | Required? | Comments |
/// |-------|--------|:---------:|----------|
/// | title | string |     ✓     |          |
/// | info  | string |           |          |
#[derive(Deserialize, Insertable)]
#[table_name = "song"]
pub struct NewSong {
    pub title: String,
    #[serde(deserialize_with = "deser_opt_string")]
    pub info: Option<String>,
}

/// The required format for the creation of new events with gigs
/// from gig requests.
///
/// ## Expected Format:
///
/// |     Field     |  Type  | Required? |              Comments              |
/// |---------------|--------|:---------:|------------------------------------|
/// | title         | string |     ✓     |                                    |
/// | info          | string |           |                                    |
/// | key           | string |           | See [Pitch](enum.Pitch.html)       |
/// | startingPitch | string |           | See [Pitch](enum.Pitch.html)       |
/// | mode          | string |           | See [SongMode](enum.SongMode.html) |
#[derive(Deserialize, AsChangeset)]
#[table_name = "song"]
pub struct SongUpdate {
    pub title: String,
    #[serde(deserialize_with = "deser_opt_string")]
    pub info: Option<String>,
    pub key: Option<Pitch>,
    #[serde(rename = "startingPitch")]
    pub starting_pitch: Option<Pitch>,
    pub mode: Option<SongMode>,
}

/// A model for songs in a setlist for an event.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE gig_song (
///   event int NOT NULL,
///   song int NOT NULL,
///   `order` int NOT NULL,
///
///   PRIMARY KEY (event, song),
///   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "event": integer,
///     "song": integer,
///     "order": integer
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "gig_song"]
#[primary_key(event, song)]
pub struct GigSong {
    /// The ID of the event this setlist refers to
    pub event: i32,
    /// The ID of the song that belongs in this setlist
    pub song: i32,
    /// When in the setlist this song appears
    pub order: i32,
}

/// The required format for creating the setlist for an event.
///
/// ## Expected Format:
///
/// | Field |  Type   | Required? | Comments |
/// |-------|---------|:---------:|----------|
/// | song  | integer |     ✓     |          |
#[derive(Deserialize)]
pub struct NewGigSong {
    pub song: i32,
}

/// The model for all types of media referenced by links on a song.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE media_type (
///   name varchar(50) NOT NULL PRIMARY KEY,
///   `order` int NOT NULL UNIQUE,
///   storage enum('local', 'remote') NOT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "order": integer,
///     "storage": string
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "media_type"]
#[primary_key(name)]
pub struct MediaType {
    /// The name of the type of media
    pub name: String,
    /// The order of where this media type appears in a song's link section
    pub order: i32,
    /// The type of storage that this type of media points to
    pub storage: StorageType,
}

/// The model for meeting minutes, or notes from officer meetings.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE minutes (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   name varchar(100) NOT NULL,
///   `date` date NOT NULL,
///   private longtext DEFAULT NULL,
///   public longtext DEFAULT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "name": string,
///     "date": date,
///     "private": string?,
///     "public": string?
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "minutes"]
pub struct MeetingMinutes {
    /// The id of the meeting minutes
    pub id: i32,
    /// The name of the meeting
    pub name: String,
    /// When these notes were initially created
    #[serde(with = "naivedate_posix")]
    pub date: NaiveDate,
    /// The private / redacted, complete officer notes
    #[serde(deserialize_with = "deser_opt_string")]
    pub private: Option<String>,
    /// The public, edited notes visible by all members
    #[serde(deserialize_with = "deser_opt_string")]
    pub public: Option<String>,
}

/// The required format for creating new meeting minutes.
///
/// ## Expected Format:
///
/// | Field |  Type  | Required? | Comments |
/// |-------|--------|:---------:|----------|
/// | name  | string |     ✓     |          |
#[derive(Insertable, Deserialize)]
#[table_name = "minutes"]
pub struct NewMeetingMinutes {
    pub name: String,
}

/// The required format for updating meeting minutes.
///
/// ## Expected Format:
///
/// |  Field  |  Type  | Required? | Comments |
/// |---------|--------|:---------:|----------|
/// | name    | string |     ✓     |          |
/// | private | string |           |          |
/// | public  | string |           |          |
#[derive(AsChangeset, Deserialize)]
#[table_name = "minutes"]
pub struct UpdatedMeetingMinutes {
    pub name: String,
    #[serde(deserialize_with = "deser_opt_string")]
    pub private: Option<String>,
    #[serde(deserialize_with = "deser_opt_string")]
    pub public: Option<String>,
}

/// The model for permissions required for site actions.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE permission (
///   name varchar(40) NOT NULL PRIMARY KEY,
///   description text DEFAULT NULL,
///   `type` enum('static', 'event') NOT NULL DEFAULT 'static'
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string,
///     "description": string?,
///     "type": string
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "permission"]
#[primary_key(name)]
pub struct Permission {
    /// The name of the permission
    pub name: String,
    /// A descriptiion of what the permission entails
    #[serde(deserialize_with = "deser_opt_string")]
    pub description: Option<String>,
    /// Whether the permission applies to a type of event or generally
    #[serde(rename = "type")]
    pub type_: PermissionType,
}

/// The model to record which members ride in which carpool for an event.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE rides_in (
///   member varchar(50) NOT NULL,
///   carpool int NOT NULL,
///
///   PRIMARY KEY (member, carpool),
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (carpool) REFERENCES carpool (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "member": string,
///     "carpool": integer
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "rides_in"]
#[primary_key(member, carpool)]
pub struct RidesIn {
    /// The email of the member in the carpool
    pub member: String,
    /// The ID of the carpool the member rides in
    pub carpool: i32,
}

/// The model that records what permissions each officer role is awarded.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE role_permission (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   role varchar(20) NOT NULL,
///   permission varchar(40) NOT NULL,
///   event_type varchar(32) DEFAULT NULL,
///
///   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (permission) REFERENCES permission (name) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (event_type) REFERENCES event_type (name) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "role": string,
///     "permission": string,
///     "eventType": string?
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "role_permission"]
pub struct RolePermission {
    /// The ID of the role permission
    pub id: i32,
    /// The name of the role this junction refers to
    pub role: String,
    /// The name of the permission the role is awarded
    pub permission: String,
    /// The type of event the permission optionally applies to
    #[serde(rename = "eventType", deserialize_with = "deser_opt_string")]
    pub event_type: Option<String>,
}

/// The model for links on a song page.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE song_link (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   song int NOT NULL,
///   `type` varchar(50) NOT NULL,
///   name varchar(128) NOT NULL,
///   target varchar(255) NOT NULL,
///
///   FOREIGN KEY (`type`) REFERENCES media_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "song": integer,
///     "type": string,
///     "name": string,
///     "target": string
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "song_link"]
pub struct SongLink {
    /// The ID of the song link
    pub id: i32,
    /// The ID of the song this link belongs to
    pub song: i32,
    /// The type of this link (e.g. MIDI)
    #[serde(rename = "type")]
    pub type_: String,
    /// The name of this link
    pub name: String,
    /// The target this link points to
    pub target: String,
}

/// The required format for creating a new song link.
///
/// ## Expected Format:
///
/// |  Field  |  Type  | Required? | Comments |
/// |---------|--------|:---------:|----------|
/// | type    | string |     ✓     |          |
/// | name    | string |     ✓     |          |
/// | target  | string |     ✓     |          |
#[derive(Deserialize)]
pub struct NewSongLink {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub target: String,
}

/// The required format for updating a song link.
///
/// ## Expected Format:
///
/// |  Field  |  Type  | Required? | Comments |
/// |---------|--------|:---------:|----------|
/// | name    | string |     ✓     |          |
/// | target  | string |     ✓     |          |
#[derive(Deserialize, AsChangeset)]
#[table_name = "song_link"]
pub struct SongLinkUpdate {
    pub name: String,
    pub target: String,
}

/// The model for tasks to do by members.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE todo (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   `text` varchar(255) NOT NULL,
///   member varchar(50) NOT NULL,
///   completed boolean NOT NULL DEFAULT '0',
///
///   FOREIGN KEY (member) REFERENCES member (email) ON UPDATE CASCADE ON DELETE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "text": string,
///     "member": string,
///     "completed": boolean
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "todo"]
pub struct Todo {
    /// The ID of this todo
    pub id: i32,
    /// The task for the member to do
    pub text: String,
    /// The email of the member that needs to complete this task
    pub member: String,
    /// Whether the task has been completed
    pub completed: bool,
}

/// The required format for creating a new todo.
///
/// ## Expected Format:
///
/// |  Field  |    Type    | Required? |         Comments          |
/// |---------|------------|:---------:|---------------------------|
/// | text    | string     |     ✓     | the task to do            |
/// | members | \[string\] |     ✓     | the emails of the members |
#[derive(Deserialize)]
pub struct NewTodo {
    pub text: String,
    pub members: Vec<String>,
}

/// The model for the types of transaction.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE transaction_type (
///   name varchar(40) NOT NULL PRIMARY KEY
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "name": string
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "transaction_type"]
#[primary_key(name)]
pub struct TransactionType {
    /// The name of the transaction type
    pub name: String,
}

/// The model for transactions between members and the club.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE transaction (
///   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
///   member varchar(50) NOT NULL,
///   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
///   amount int NOT NULL,
///   description varchar(500) NOT NULL,
///   semester varchar(32) DEFAULT NULL,
///   `type` varchar(40) NOT NULL,
///   resolved tinyint(1) NOT NULL DEFAULT '0',
///
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (`type`) REFERENCES transaction_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
///   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "id": integer,
///     "member": string,
///     "time": datetime,
///     "amount": integer,
///     "description": string,
///     "semester": string?,
///     "type": string,
///     "resolved": boolean
/// }
/// ```
#[derive(Serialize, Identifiable, Queryable)]
#[table_name = "transaction"]
pub struct Transaction {
    /// The ID of the transaction
    pub id: i32,
    /// The email of the member this transaction was charged to
    pub member: String,
    /// When this transaction was charged
    #[serde(with = "naivedatetime_posix")]
    pub time: NaiveDateTime,
    /// How much this transaction was for
    pub amount: i32,
    /// A description of what the member was charged for specifically
    pub description: String,
    /// Optionally, the name of the semester this transaction was made during
    #[serde(deserialize_with = "deser_opt_string")]
    pub semester: Option<String>,
    /// The name of the type of transaction
    #[serde(rename = "type")]
    pub type_: String,
    /// Whether the member has paid the amount requested in this transaction
    pub resolved: bool,
}

/// The required format for creating a new todo.
///
/// ## Expected Format:
///
/// |    Field    |  Type   | Required? | Comments |
/// |-------------|---------|:---------:|----------|
/// | member      | string  |     ✓     |          |
/// | amount      | integer |     ✓     |          |
/// | description | string  |     ✓     |          |
/// | semester    | string  |           |          |
/// | type        | string  |     ✓     |          |
/// | resolved    | boolean |     ✓     |          |
#[derive(Serialize, Insertable, Queryable)]
#[table_name = "transaction"]
pub struct NewTransaction {
    pub member: String,
    pub amount: i32,
    pub description: String,
    #[serde(deserialize_with = "deser_opt_string")]
    pub semester: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub resolved: bool,
}

/// The model for login sessions for members.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE session (
///   member varchar(50) NOT NULL PRIMARY KEY,
///   `key` varchar(64) NOT NULL,
///
///   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "member": string,
///     "key": string
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize, Insertable)]
#[table_name = "session"]
#[primary_key(member)]
pub struct Session {
    /// The email of the logged in member
    pub member: String,
    /// The login token unique to the member
    pub key: String,
}

/// The required format for logging in.
///
/// ## Expected Format:
///
/// |  Field   |  Type  | Required? | Comments |
/// |----------|--------|:---------:|----------|
/// | email    | string |     ✓     |          |
/// | passHash | string |     ✓     |          |
#[derive(Deserialize)]
pub struct LoginInfo {
    pub email: String,
    #[serde(rename = "passHash")]
    pub pass_hash: String,
}

/// The model for any other variables needed by the API.
///
/// ## Database Format:
///
/// ```sql
/// CREATE TABLE variable (
///   `key` varchar(255) NOT NULL PRIMARY KEY,
///   value varchar(255) NOT NULL
/// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/// ```
///
/// ## JSON Format:
///
/// ```json
/// {
///     "key": string,
///     "value": string
/// }
/// ```
#[derive(Identifiable, Queryable, Serialize, Deserialize, Insertable)]
#[table_name = "variable"]
#[primary_key(key)]
pub struct Variable {
    /// The name of the variable
    pub key: String,
    /// The value of the variable
    pub value: String,
}

/// The required format for setting a variable.
///
/// ## Expected Format:
///
/// | Field |  Type  | Required? | Comments |
/// |-------|--------|:---------:|----------|
/// | value | string |     ✓     |          |
#[derive(Deserialize)]
pub struct NewValue {
    pub value: String,
}

/// Deserialize an Option<String> normally, but Some("") maps to None.
fn deser_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|s| s.filter(|s| s.len() > 0))
}

/// Deserialize an Option<Enrollment>.
fn deser_enrollment<'de, D>(deserializer: D) -> Result<Option<Enrollment>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(s) = Option::<String>::deserialize(deserializer)? {
        match s.as_str() {
            "" | "inactive" => Ok(None),
            "class" => Ok(Some(Enrollment::Class)),
            "club" => Ok(Some(Enrollment::Club)),
            other => Err(D::Error::invalid_value(
                Unexpected::Str(other),
                &"\"inactive\", \"class\", \"club\", or null",
            )),
        }
    } else {
        Ok(None)
    }
}

pub mod naivedatetime_posix {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.timestamp() * 1000)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer).map(|posix| NaiveDateTime::from_timestamp(posix / 1000, 0))
    }
}

pub mod optional_naivedatetime_posix {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => serializer.serialize_i64(dt.timestamp() * 1000),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<i64>::deserialize(deserializer)
            .map(|posix| posix.map(|p| NaiveDateTime::from_timestamp(p / 1000, 0)))
    }
}

pub mod naivedate_posix {
    use chrono::{NaiveDate, NaiveDateTime};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.and_hms(0, 0, 0).timestamp() * 1000)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer)
            .map(|posix| NaiveDateTime::from_timestamp(posix / 1000, 0).date())
    }
}

pub mod optional_naivedate_posix {
    use chrono::{NaiveDate, NaiveDateTime};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt {
            Some(dt) => serializer.serialize_i64(dt.and_hms(0, 0, 0).timestamp() * 1000),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<i64>::deserialize(deserializer)
            .map(|posix| posix.map(|p| NaiveDateTime::from_timestamp(p / 1000, 0).date()))
    }
}
