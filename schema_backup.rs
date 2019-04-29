#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum AbsenceRequestState {
    Approved,
    Denied,
    Pending,
}

table! {
    use super::AbsenceRequestStateMapping;
    use diesel::sql_types::*;

    absence_request (member, event) {
        member -> Varchar,
        event -> Integer,
        time -> Timestamp,
        reason -> Varchar,
        state -> AbsenceRequestStateMapping,
    }
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum Enrollment {
    Class,
    Club,
}

table! {
    use super::EnrollmentMapping;
    use diesel::sql_types::*;

    active_semester (member, semester, choir) {
        member -> Varchar,
        semester -> Integer,
        choir -> Varchar,
        enrollment -> EnrollmentMapping,
        section -> Nullable<Integer>,
    }
}

table! {
    announcement (id) {
        id -> Integer,
        choir -> Varchar,
        member -> Nullable<Varchar>,
        semester -> Integer,
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
        did_attend -> Nullable<Bool>,
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
    choir (name) {
        name -> Varchar,
        officer_email_list -> Varchar,
        member_email_list -> Varchar,
    }
}

table! {
    event (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Varchar,
        semester -> Integer,
        #[sql_name = "type"]
        type_ -> Integer,
        call_time -> Datetime,
        release_time -> Nullable<Datetime>,
        points -> Integer,
        comments -> Nullable<Text>,
        location -> Nullable<Varchar>,
        gig_count -> Bool,
        default_attend -> Bool,
        section -> Nullable<Integer>,
    }
}

table! {
    event_type (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Varchar,
        weight -> Integer,
    }
}

table! {
    fee (name, choir) {
        name -> Varchar,
        choir -> Varchar,
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

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum GigRequestStatus {
    Accepted,
    Dismissed,
    Pending,
}

table! {
    use super::GigRequestStatusMapping;
    use diesel::sql_types::*;

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
    google_docs (name, choir) {
        name -> Varchar,
        choir -> Varchar,
        url -> Varchar,
    }
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum StorageType {
    Local,
    Remote,
}

table! {
    use super::StorageTypeMapping;
    use diesel::sql_types::*;

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
        phone -> Varchar,
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
        role -> Integer,
        semester -> Integer,
    }
}

table! {
    minutes (id) {
        id -> Integer,
        choir -> Varchar,
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
        choir -> Varchar,
    }
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum BorrowStatus {
    Circulating,
    Lost,
    Decommissioned,
}

table! {
    use super::BorrowStatusMapping;
    use diesel::sql_types::*;

    outfit_borrow (outfit) {
        outfit -> Integer,
        member -> Varchar,
        status -> BorrowStatusMapping,
    }
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum PermissionType {
    Static,
    Event,
}

table! {
    use super::PermissionTypeMapping;
    use diesel::sql_types::*;

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
    role (id) {
        id -> Integer,
        name -> Nullable<Varchar>,
        choir -> Varchar,
        rank -> Integer,
        max_quantity -> Integer,
    }
}

table! {
    role_permission (id) {
        id -> Integer,
        role -> Integer,
        permission -> Varchar,
        event_type -> Nullable<Integer>,
    }
}

table! {
    section_type (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Nullable<Varchar>,
    }
}

table! {
    semester (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Varchar,
        start_date -> Datetime,
        end_date -> Datetime,
        gig_requirement -> Integer,
    }
}

table! {
    session (member) {
        member -> Varchar,
        key -> Varchar,
    }
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum Key {
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
    BSharp,
    CFlat,
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    ESharp,
    FFlat,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
}

#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq)]
pub enum SongMode {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

table! {
    use super::KeyMapping;
    use super::SongModeMapping;
    use diesel::sql_types::*;

    song (id) {
        id -> Integer,
        choir -> Varchar,
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
        choir -> Varchar,
        time -> Timestamp,
        amount -> Integer,
        description -> Varchar,
        semester -> Nullable<Integer>,
        #[sql_name = "type"]
        type_ -> Nullable<Integer>,
        resolved -> Bool,
    }
}

table! {
    transaction_type (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Varchar,
    }
}

table! {
    uniform (id) {
        id -> Integer,
        name -> Varchar,
        choir -> Varchar,
        description -> Nullable<Text>,
    }
}

table! {
    variable (choir, key) {
        choir -> Varchar,
        key -> Varchar,
        value -> Varchar,
    }
}

joinable!(absence_request -> event (event));
joinable!(absence_request -> member (member));
joinable!(active_semester -> choir (choir));
joinable!(active_semester -> member (member));
joinable!(active_semester -> section_type (section));
joinable!(active_semester -> semester (semester));
joinable!(announcement -> choir (choir));
joinable!(announcement -> member (member));
joinable!(announcement -> semester (semester));
joinable!(attendance -> event (event));
joinable!(attendance -> member (member));
joinable!(carpool -> event (event));
joinable!(carpool -> member (driver));
joinable!(event -> choir (choir));
joinable!(event -> event_type (type));
joinable!(event -> section_type (section));
joinable!(event -> semester (semester));
joinable!(event_type -> choir (choir));
joinable!(fee -> choir (choir));
joinable!(gig -> event (event));
joinable!(gig -> uniform (uniform));
joinable!(gig_request -> event (event));
joinable!(gig_song -> event (event));
joinable!(gig_song -> song (song));
joinable!(google_docs -> choir (choir));
joinable!(member_role -> member (member));
joinable!(member_role -> role (role));
joinable!(member_role -> semester (semester));
joinable!(minutes -> choir (choir));
joinable!(outfit -> choir (choir));
joinable!(outfit_borrow -> member (member));
joinable!(outfit_borrow -> outfit (outfit));
joinable!(rides_in -> carpool (carpool));
joinable!(rides_in -> member (member));
joinable!(role -> choir (choir));
joinable!(role_permission -> event_type (event_type));
joinable!(role_permission -> permission (permission));
joinable!(role_permission -> role (role));
joinable!(section_type -> choir (choir));
joinable!(semester -> choir (choir));
joinable!(session -> member (member));
joinable!(song -> choir (choir));
joinable!(song_link -> media_type (type));
joinable!(song_link -> song (song));
joinable!(todo -> member (member));
joinable!(transaction -> choir (choir));
joinable!(transaction -> member (member));
joinable!(transaction -> semester (semester));
joinable!(transaction -> transaction_type (type));
joinable!(transaction_type -> choir (choir));
joinable!(uniform -> choir (choir));
joinable!(variable -> choir (choir));

allow_tables_to_appear_in_same_query!(
    absence_request,
    active_semester,
    announcement,
    attendance,
    carpool,
    choir,
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
