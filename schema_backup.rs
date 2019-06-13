use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

pub mod enums {
    pub use super::AbsenceRequestState;
    pub use super::BorrowStatus;
    pub use super::Enrollment;
    pub use super::GigRequestStatus;
    pub use super::Key;
    pub use super::PermissionType;
    pub use super::SongMode;
    pub use super::StorageType;
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum AbsenceRequestState {
    Pending,
    Approved,
    Denied,
}

table! {
    use diesel::sql_types::*;
    use super::AbsenceRequestStateMapping;

    absence_request (member, event) {
        member -> Varchar,
        event -> Integer,
        time -> Timestamp,
        reason -> Varchar,
        state -> AbsenceRequestStateMapping,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum Enrollment {
    Class,
    Club,
}

table! {
    use diesel::sql_types::*;
    use super::EnrollmentMapping;

    active_semester (member, semester) {
        member -> Varchar,
        semester -> Varchar,
        enrollment -> EnrollmentMapping,
        section -> Nullable<Varchar>,
    }
}

table! {
    announcement (id) {
        id -> Integer,
        member -> Nullable<Varchar>,
        semester -> Varchar,
        time -> Timestamp,
        content -> Longtext,
        archived -> Bool,
    }
}

table! {
    attendance (member, event) {
        member -> Varchar,
        event -> Integer,
        should_attend -> Bool,
        did_attend -> Bool,
        confirmed -> Bool,
        minutes_late -> Integer,
    }
}

table! {
    carpool (id) {
        id -> Integer,
        event -> Integer,
        driver -> Varchar,
    }
}

table! {
    event (id) {
        id -> Integer,
        name -> Varchar,
        semester -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        call_time -> Datetime,
        release_time -> Nullable<Datetime>,
        points -> Integer,
        comments -> Nullable<Text>,
        location -> Nullable<Varchar>,
        gig_count -> Bool,
        default_attend -> Bool,
        section -> Nullable<Varchar>,
    }
}

table! {
    event_type (name) {
        name -> Varchar,
        weight -> Integer,
    }
}

table! {
    fee (name) {
        name -> Varchar,
        amount -> Integer,
    }
}

table! {
    gig (event) {
        event -> Integer,
        performance_time -> Datetime,
        uniform -> Integer,
        contact_name -> Nullable<Varchar>,
        contact_email -> Nullable<Varchar>,
        contact_phone -> Nullable<Varchar>,
        price -> Nullable<Integer>,
        public -> Bool,
        summary -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum GigRequestStatus {
    Pending,
    Accepted,
    Dismissed,
}

table! {
    use diesel::sql_types::*;
    use super::GigRequestStatusMapping;

    gig_request (id) {
        id -> Integer,
        time -> Timestamp,
        name -> Varchar,
        organization -> Varchar,
        event -> Nullable<Integer>,
        contact_name -> Varchar,
        contact_phone -> Varchar,
        contact_email -> Varchar,
        start_time -> Datetime,
        location -> Varchar,
        comments -> Nullable<Text>,
        status -> GigRequestStatusMapping,
    }
}

table! {
    gig_song (id) {
        id -> Integer,
        event -> Integer,
        song -> Integer,
        order -> Integer,
    }
}

table! {
    google_docs (name) {
        name -> Varchar,
        url -> Varchar,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    Remote,
}

table! {
    use diesel::sql_types::*;
    use super::StorageTypeMapping;

    media_type (name) {
        name -> Varchar,
        order -> Integer,
        storage -> StorageTypeMapping,
    }
}

table! {
    member (email) {
        email -> Varchar,
        first_name -> Varchar,
        preferred_name -> Nullable<Varchar>,
        last_name -> Varchar,
        pass_hash -> Varchar,
        phone_number -> Varchar,
        picture -> Nullable<Varchar>,
        passengers -> Integer,
        location -> Varchar,
        about -> Nullable<Varchar>,
        major -> Nullable<Varchar>,
        minor -> Nullable<Varchar>,
        hometown -> Nullable<Varchar>,
        arrived_at_tech -> Nullable<Integer>,
        gateway_drug -> Nullable<Varchar>,
        conflicts -> Nullable<Varchar>,
        dietary_restrictions -> Nullable<Varchar>,
    }
}

table! {
    member_role (member, role, semester) {
        member -> Varchar,
        role -> Varchar,
        semester -> Varchar,
    }
}

table! {
    minutes (id) {
        id -> Integer,
        name -> Varchar,
        date -> Date,
        private -> Nullable<Longtext>,
        public -> Nullable<Longtext>,
    }
}

table! {
    outfit (id) {
        id -> Integer,
        name -> Varchar,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum BorrowStatus {
    Circulating,
    Lost,
    Decommissioned,
}

table! {
    use diesel::sql_types::*;
    use super::BorrowStatusMapping;

    outfit_borrow (outfit) {
        outfit -> Integer,
        member -> Varchar,
        status -> BorrowStatusMapping,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum PermissionType {
    Static,
    Event,
}

table! {
    use diesel::sql_types::*;
    use super::PermissionTypeMapping;

    permission (name) {
        name -> Varchar,
        description -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> PermissionTypeMapping,
    }
}

table! {
    rides_in (member, carpool) {
        member -> Varchar,
        carpool -> Integer,
    }
}

table! {
    role (name) {
        name -> Varchar,
        rank -> Integer,
        max_quantity -> Integer,
    }
}

table! {
    role_permission (role, permission) {
        role -> Varchar,
        permission -> Varchar,
        event_type -> Nullable<Varchar>,
    }
}

table! {
    section_type (name) {
        name -> Varchar,
    }
}

table! {
    semester (name) {
        name -> Varchar,
        start_date -> Datetime,
        end_date -> Datetime,
        gig_requirement -> Integer,
        current -> Bool,
    }
}

table! {
    session (member) {
        member -> Varchar,
        key -> Varchar,
    }
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum Key {
    #[db_rename = "A\x26\x6D"]
    AFlat,
    #[db_rename = "A"]
    A,
    #[db_rename = "A#"]
    ASharp,
    #[db_rename = "B\x26\x6D"]
    BFlat,
    #[db_rename = "B"]
    B,
    #[db_rename = "B#"]
    BSharp,
    #[db_rename = "C\x26\x6D"]
    CFlat,
    #[db_rename = "C"]
    C,
    #[db_rename = "C#"]
    CSharp,
    #[db_rename = "D\x26\x6D"]
    DFlat,
    #[db_rename = "D"]
    D,
    #[db_rename = "D#"]
    DSharp,
    #[db_rename = "E\x26\x6D"]
    EFlat,
    #[db_rename = "E"]
    E,
    #[db_rename = "E#"]
    ESharp,
    #[db_rename = "F\x26\x6D"]
    FFlat,
    #[db_rename = "F"]
    F,
    #[db_rename = "F#"]
    FSharp,
    #[db_rename = "G\x26\x6D"]
    GFlat,
    #[db_rename = "G"]
    G,
    #[db_rename = "G#"]
    GSharp,
}

#[derive(Debug, PartialEq, DbEnum, Serialize, Deserialize)]
pub enum SongMode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Myxolydian,
    Locrian,
}

table! {
    use diesel::sql_types::*;
    use super::KeyMapping;
    use super::SongModeMapping;

    song (id) {
        id -> Integer,
        title -> Varchar,
        info -> Nullable<Text>,
        current -> Bool,
        key -> Nullable<KeyMapping>,
        starting_pitch -> Nullable<KeyMapping>,
        mode -> Nullable<SongModeMapping>,
    }
}

table! {
    song_link (id) {
        id -> Integer,
        song -> Integer,
        #[sql_name = "type"]
        type_ -> Varchar,
        name -> Varchar,
        target -> Varchar,
    }
}

table! {
    todo (id) {
        id -> Integer,
        text -> Varchar,
        member -> Varchar,
        completed -> Bool,
    }
}

table! {
    transaction (id) {
        id -> Integer,
        member -> Varchar,
        time -> Timestamp,
        amount -> Integer,
        description -> Varchar,
        semester -> Nullable<Varchar>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        resolved -> Bool,
    }
}

table! {
    transaction_type (name) {
        name -> Varchar,
    }
}

table! {
    uniform (id) {
        id -> Integer,
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

table! {
    variable (key) {
        key -> Varchar,
        value -> Varchar,
    }
}

joinable!(absence_request -> event (event));
joinable!(absence_request -> member (member));
joinable!(active_semester -> member (member));
joinable!(active_semester -> section_type (section));
joinable!(active_semester -> semester (semester));
joinable!(announcement -> member (member));
joinable!(announcement -> semester (semester));
joinable!(attendance -> event (event));
joinable!(attendance -> member (member));
joinable!(carpool -> event (event));
joinable!(carpool -> member (driver));
joinable!(event -> event_type (type_));
joinable!(event -> section_type (section));
joinable!(event -> semester (semester));
joinable!(gig -> event (event));
joinable!(gig -> uniform (uniform));
joinable!(gig_request -> event (event));
joinable!(gig_song -> event (event));
joinable!(gig_song -> song (song));
joinable!(member_role -> member (member));
joinable!(member_role -> role (role));
joinable!(member_role -> semester (semester));
joinable!(outfit_borrow -> member (member));
joinable!(outfit_borrow -> outfit (outfit));
joinable!(rides_in -> carpool (carpool));
joinable!(rides_in -> member (member));
joinable!(role_permission -> event_type (event_type));
joinable!(role_permission -> permission (permission));
joinable!(role_permission -> role (role));
joinable!(session -> member (member));
joinable!(song_link -> media_type (type_));
joinable!(song_link -> song (song));
joinable!(todo -> member (member));
joinable!(transaction -> member (member));
joinable!(transaction -> semester (semester));
joinable!(transaction -> transaction_type (type_));

allow_tables_to_appear_in_same_query!(
    absence_request,
    active_semester,
    announcement,
    attendance,
    carpool,
    event,
    event_type,
    fee,
    gig,
    gig_request,
    gig_song,
    google_docs,
    media_type,
    member,
    member_role,
    minutes,
    outfit,
    outfit_borrow,
    permission,
    rides_in,
    role,
    role_permission,
    section_type,
    semester,
    session,
    song,
    song_link,
    todo,
    transaction,
    transaction_type,
    uniform,
    variable,
);
