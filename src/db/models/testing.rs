extern crate lazy_static;

use self::lazy_static::lazy_static;
use super::member::MemberForSemester;
use super::*;

lazy_static! {
    pub static ref SECTION_TYPES: [SectionType; 4] = [
        SectionType {
            name: "Bass".to_owned(),
        },
        SectionType {
            name: "Baritone".to_owned(),
        },
        SectionType {
            name: "Tenor 2".to_owned(),
        },
        SectionType {
            name: "Tenor 1".to_owned(),
        },
    ];
    pub static ref MEMBERS: [Member; 3] = [
        Member {
            email: "theswordguy@gmail.com".to_owned(),
            first_name: "Cloud".to_owned(),
            preferred_name: None,
            last_name: "Strife".to_owned(),
            pass_hash: "aeris123".to_owned(),
            phone_number: "8005882300".to_owned(),
            picture: None,
            passengers: 2,
            location: "Midgar".to_owned(),
            on_campus: None,
            about: None,
            major: Some("Slashing".to_owned()),
            minor: Some("Dashing".to_owned()),
            hometown: Some("Nibelheim".to_owned()),
            arrived_at_tech: Some(2),
            gateway_drug: Some("Materia".to_owned()),
            conflicts: None,
            dietary_restrictions: None,
        },
        Member {
            email: "aerith@gmail.com".to_owned(),
            first_name: "Aerith".to_owned(),
            preferred_name: Some("Aeris".to_owned()),
            last_name: "Gainsborough".to_owned(),
            pass_hash: "illneverdie".to_owned(),
            phone_number: "1234567890".to_owned(),
            picture: None,
            passengers: 0,
            location: "Church".to_owned(),
            on_campus: Some(false),
            about: None,
            major: Some("Business".to_owned()),
            minor: None,
            hometown: None,
            arrived_at_tech: Some(1),
            gateway_drug: None,
            conflicts: None,
            dietary_restrictions: Some("vegetarian".to_owned()),
        },
        Member {
            email: "reddeadredemption@yahoo.com".to_owned(),
            first_name: "Vincent".to_owned(),
            preferred_name: Some("Vinny".to_owned()),
            last_name: "Valentine".to_owned(),
            pass_hash: "metallica".to_owned(),
            phone_number: "4242564000".to_owned(),
            picture: None,
            passengers: 3,
            location: "Shinra".to_owned(),
            on_campus: Some(true),
            about: None,
            major: Some("Gungineering".to_owned()),
            minor: None,
            hometown: Some("the ground".to_owned()),
            arrived_at_tech: None,
            gateway_drug: None,
            conflicts: Some("internal".to_owned()),
            dietary_restrictions: Some("metal".to_owned()),
        },
    ];
    pub static ref SEMESTERS: [Semester; 2] = [
        Semester {
            name: "Spring 2016".to_owned(),
            start_date: NaiveDateTime::parse_from_str("12:30:00 PM 2016-01-11", "%r %F").unwrap(),
            end_date: NaiveDateTime::parse_from_str("1:30:00 PM 2016-05-03", "%r %F").unwrap(),
            gig_requirement: 5,
            current: true,
        },
        Semester {
            name: "Fall 2015".to_owned(),
            start_date: NaiveDateTime::parse_from_str("12:30:00 PM 2015-08-12", "%r %F").unwrap(),
            end_date: NaiveDateTime::parse_from_str("1:30:00 PM 2015-12-05", "%r %F").unwrap(),
            gig_requirement: 4,
            current: false,
        },
    ];
    pub static ref MEMBERS_FOR_SEMESTERS: [MemberForSemester; 2] = [
        MemberForSemester {
            member: MEMBERS[0].clone(),
            active_semester: Some(ActiveSemester {
                member: MEMBERS[0].email.clone(),
                semester: SEMESTERS[0].name.clone(),
                enrollment: Enrollment::Club,
                section: Some(SECTION_TYPES[1].name.clone()),
            }),
        },
        MemberForSemester {
            member: MEMBERS[1].clone(),
            active_semester: Some(ActiveSemester {
                member: MEMBERS[1].email.clone(),
                semester: SEMESTERS[0].name.clone(),
                enrollment: Enrollment::Class,
                section: Some(SECTION_TYPES[3].name.clone()),
            }),
        },
    ];
    pub static ref EVENT_TYPES: [EventType; 6] = [
        EventType {
            name: "Rehearsal".to_owned(),
            weight: 10,
        },
        EventType {
            name: "Sectional".to_owned(),
            weight: 5,
        },
        EventType {
            name: "Tutti".to_owned(),
            weight: 35,
        },
        EventType {
            name: "Ombuds".to_owned(),
            weight: 5,
        },
        EventType {
            name: "Volunteer".to_owned(),
            weight: 10,
        },
        EventType {
            name: "Other".to_owned(),
            weight: 5,
        },
    ];
    pub static ref EVENTS: [Event; 4] = [
        Event {
            id: 1,
            name: "Weekly Rehearsal".to_owned(),
            semester: SEMESTERS[0].name.clone(),
            type_: EVENT_TYPES[0].name.clone(),
            call_time: NaiveDateTime::parse_from_str("06:00:00 PM 2016-01-18", "%r %F").unwrap(),
            release_time: Some(
                NaiveDateTime::parse_from_str("08:00:00 PM 2016-01-18", "%r %F").unwrap()
            ),
            points: EVENT_TYPES[0].weight,
            comments: None,
            location: Some("Willage 175".to_owned()),
            gig_count: false,
            default_attend: true,
            section: None,
        },
        Event {
            id: 2,
            name: "Flash Mob!".to_owned(),
            semester: SEMESTERS[0].name.clone(),
            type_: EVENT_TYPES[4].name.clone(),
            call_time: NaiveDateTime::parse_from_str("01:30:00 PM 2016-01-21", "%r %F").unwrap(),
            release_time: None,
            points: EVENT_TYPES[4].weight,
            comments: Some(
                "Come to the skiles walkway after lunch so we can spread the Glub word!".to_owned()
            ),
            location: Some("Skile Walkway".to_owned()),
            gig_count: true,
            default_attend: true,
            section: None,
        },
        Event {
            id: 3,
            name: "Weekly Sectional".to_owned(),
            semester: SEMESTERS[0].name.clone(),
            type_: EVENT_TYPES[1].name.clone(),
            call_time: NaiveDateTime::parse_from_str("06:00:00 PM 2016-01-21", "%r %F").unwrap(),
            release_time: Some(
                NaiveDateTime::parse_from_str("07:00:00 PM 2016-01-21", "%r %F").unwrap()
            ),
            points: EVENT_TYPES[1].weight,
            comments: None,
            location: Some("Willage 175".to_owned()),
            gig_count: false,
            default_attend: true,
            section: None,
        },
        Event {
            id: 4,
            name: "Eddie's Attic".to_owned(),
            semester: SEMESTERS[0].name.clone(),
            type_: EVENT_TYPES[2].name.clone(),
            call_time: NaiveDateTime::parse_from_str("05:30:00 PM 2016-04-20", "%r %F").unwrap(),
            release_time: Some(
                NaiveDateTime::parse_from_str("09:00:00 PM 2016-04-20", "%r %F").unwrap()
            ),
            points: EVENT_TYPES[2].weight,
            comments: Some(
                "It's the best reason to not drop out yet! \
                 Check the setlist for what to have memorized."
                    .to_owned()
            ),
            location: Some("Eddie's Attic in Decatur, GA".to_owned()),
            gig_count: false,
            default_attend: true,
            section: None,
        },
    ];
    pub static ref ATTENDANCE: [Attendance; 8] = [
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[0].member.email.clone(),
            event: EVENTS[0].id,
            should_attend: EVENTS[0].default_attend,
            did_attend: true,
            confirmed: false,
            minutes_late: 0,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[0].member.email.clone(),
            event: EVENTS[1].id,
            should_attend: EVENTS[1].default_attend,
            did_attend: true,
            confirmed: false,
            minutes_late: 5,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[0].member.email.clone(),
            event: EVENTS[2].id,
            should_attend: EVENTS[2].default_attend,
            did_attend: false,
            confirmed: false,
            minutes_late: 0,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[0].member.email.clone(),
            event: EVENTS[3].id,
            should_attend: EVENTS[3].default_attend,
            did_attend: true,
            confirmed: true,
            minutes_late: 10,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[1].member.email.clone(),
            event: EVENTS[0].id,
            should_attend: EVENTS[0].default_attend,
            did_attend: true,
            confirmed: true,
            minutes_late: 0,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[1].member.email.clone(),
            event: EVENTS[1].id,
            should_attend: EVENTS[1].default_attend,
            did_attend: true,
            confirmed: true,
            minutes_late: 0,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[1].member.email.clone(),
            event: EVENTS[2].id,
            should_attend: EVENTS[2].default_attend,
            did_attend: false,
            confirmed: true,
            minutes_late: 0,
        },
        Attendance {
            member: MEMBERS_FOR_SEMESTERS[1].member.email.clone(),
            event: EVENTS[3].id,
            should_attend: EVENTS[3].default_attend,
            did_attend: true,
            confirmed: true,
            minutes_late: 0,
        },
    ];
    pub static ref ABSENCE_REQUESTS: [AbsenceRequest; 1] = [
        AbsenceRequest {
            member: MEMBERS_FOR_SEMESTERS[1].member.email.clone(),
            event: EVENTS[2].id,
            time: NaiveDateTime::parse_from_str("10:03:44 AM 2016-01-19", "%r %F").unwrap(), // two days early
            reason: "I have a doctor's appointment that I forgot about, sorry.".to_owned(),
            state: AbsenceRequestState::Approved,
        },
    ];
    pub static ref ANNOUNCEMENTS: [Announcement; 2] = [
        Announcement {
            id: 1,
            member: None,
            semester: SEMESTERS[1].name.clone(),
            time: NaiveDateTime::parse_from_str("01:05:20 PM 2015-09-02", "%r %F").unwrap(),
            content: "Don't forget to pack for retreat next weekend!".to_owned(),
            archived: true,
        },
        Announcement {
            id: 2,
            member: Some(MEMBERS_FOR_SEMESTERS[1].member.email.clone()),
            semester: SEMESTERS[0].name.clone(),
            time: NaiveDateTime::parse_from_str("02:31:38 PM 2016-02-24", "%r %F").unwrap(),
            content: "Please be sure to bring a pencil and binder to every rehearsal.".to_owned(),
            archived: false,
        },
    ];
    pub static ref CARPOOLS: [Carpool; 1] = [
        Carpool {
            id: 5,
            event: EVENTS[3].id,
            driver: MEMBERS[1].email.clone(),
        },
    ];
    pub static ref RIDES_INS: [RidesIn; 2] = [
        RidesIn {
            member: MEMBERS[0].email.clone(),
            carpool: CARPOOLS[0].id,
        },
        RidesIn {
            member: MEMBERS[2].email.clone(),
            carpool: CARPOOLS[0].id,
        },
    ];
    pub static ref FEES: [Fee; 3] = [
        Fee {
            name: "dues".to_owned(),
            description: "Semester Dues".to_owned(),
            amount: 20,
        },
        Fee {
            name: "latedues".to_owned(),
            description: "Dues Late Fee".to_owned(),
            amount: 5,
        },
        Fee {
            name: "ties".to_owned(),
            description: "Ties Deposit".to_owned(),
            amount: 5,
        },
    ];
    pub static ref GOOGLE_DOCS: [GoogleDoc; 2] = [
        GoogleDoc {
            name: "Handbook".to_owned(),
            url: "https://www.google.com/document1".to_owned(),
        },
        GoogleDoc {
            name: "Constitution".to_owned(),
            url: "https://www.google.com/document2".to_owned(),
        },
    ];
    pub static ref UNIFORMS: [Uniform; 6] = [
        Uniform {
            id: 1,
            name: "Casual".to_owned(),
            description: Some("Anything goes. Preferably including pants, shirt, \
                               and shoes. Underwear optional.".to_owned()),
            color: Some("#a8c".to_owned()),
        },
        Uniform {
            id: 2,
            name: "Jeans Mode".to_owned(),
            description: Some("White, long-sleeved, button-up shirt -- IRONED. \
                               Dark-wash blue jeans. Completely black belt. \
                               Glee Club tie. Black shoes, preferably on the formal \
                               side. Non-white socks. An undershirt is a good idea.".to_owned()),
            color: Some("#137".to_owned()),
        },
        Uniform {
            id: 3,
            name: "GT/GC".to_owned(),
            description: Some("Ironed white long-sleeved button-up shirt. \
                               Glee Club tie. Black slacks. Black dress shoes. \
                               Black belt. An undershirt is a good idea. \
                               Depending on the circumstances, a suit \
                               jacket may be acceptable.".to_owned()),
            color: Some("#dc3".to_owned()),
        },
        Uniform {
            id: 4,
            name: "GT/GC Casual".to_owned(),
            description: Some("Casual, but wear the Glee Club T-shirt if you \
                               have it. Otherwise, wear a different GT T-shirt \
                               or a gold-ish T-shirt.".to_owned()),
            color: Some("#dc3".to_owned()),
        },
        Uniform {
            id: 5,
            name: "T-Shirt Mode".to_owned(),
            description: Some("Wear your GC t-shirt with dark wash jeans. \
                               Shoes/belt can be whatever color.".to_owned()),
            color: Some("#137".to_owned()),
        },
        Uniform {
            id: 6,
            name: "Wedding Mode".to_owned(),
            description: Some("Full suit and tie.".to_owned()),
            color: Some("#000".to_owned()),
        },
    ];
    pub static ref GIGS: [Gig; 1] = [
        Gig {
            event: EVENTS[3].id,
            performance_time: NaiveDateTime::parse_from_str("07:00:00 PM 2016-04-20", "%r %F").unwrap(),
            uniform: UNIFORMS[1].id,
            contact_name: None,
            contact_email: None,
            contact_phone: None,
            price: Some(0),
            public: true,
            summary: Some("Glee Club at Eddie's Attic".to_owned()),
            description: Some("The Glee Club is singing at Eddie's Attic! \
                               Join us to enjoy the atmosphere and get \
                               your face melted.".to_owned()),
        },
    ];
    pub static ref GIG_REQUESTS: [GigRequest; 1] = [
        GigRequest {
            id: 3,
            time: NaiveDateTime::parse_from_str("08:21:30 PM 2016-03-10", "%r %F").unwrap(),
            name: "My Wedding!".to_owned(),
            organization: "Tifa Lockhart".to_owned(),
            event: None,
            contact_name: "Tifa Lockhart".to_owned(),
            contact_email: "punch@it.org".to_owned(),
            contact_phone: "2223334444".to_owned(),
            start_time: NaiveDateTime::parse_from_str("05:30:00 PM 2016-04-28", "%r %F").unwrap(),
            location: "Dahlonega".to_owned(),
            comments: None,
            status: GigRequestStatus::Pending,
        },
    ];
    pub static ref SONGS: [Song; 3] = [
        Song {
            id: 1,
            title: "Ramblin' Wreck".to_owned(),
            info: Some("Our fight song!".to_owned()),
            current: true,
            key: Some(Key::C),
            starting_pitch: Some(Key::G),
            mode: Some(SongMode::Major),
        },
        Song {
            id: 2,
            title: "Eagles Medley".to_owned(),
            info: Some("A mashup of 'Seven Bridges Road' and 'Take it Easy', both by the Eagles.".to_owned()),
            current: true,
            key: Some(Key::D),
            starting_pitch: Some(Key::D),
            mode: Some(SongMode::Major),
        },
        Song {
            id: 3,
            title: "It's Raining Men".to_owned(),
            info: Some("You know this song.".to_owned()),
            current: false,
            key: Some(Key::E),
            starting_pitch: Some(Key::E),
            mode: Some(SongMode::Minor),
        },
    ];
    pub static ref SONG_LINKS: [SongLink; 5] = [
        SongLink {
            id: 1,
            song: SONGS[0].id,
            type_: MEDIA_TYPES[2].name.clone(),
            name: "TTBB Sheet Music".to_owned(),
            target: "Ramble%20TTBB%20Sheet%20Music.pdf".to_owned(),
        },
        SongLink {
            id: 2,
            song: SONGS[0].id,
            type_: MEDIA_TYPES[3].name.clone(),
            name: "Live Performance".to_owned(),
            target: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_owned(),
        },
        SongLink {
            id: 3,
            song: SONGS[1].id,
            type_: MEDIA_TYPES[2].name.clone(),
            name: "TTBB Sheet Music".to_owned(),
            target: "Eagles%20TTBB%20Sheet%20Music.pdf".to_owned(),
        },
        SongLink {
            id: 4,
            song: SONGS[1].id,
            type_: MEDIA_TYPES[3].name.clone(),
            name: "Live at Eddie's".to_owned(),
            target: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_owned(),
        },
        SongLink {
            id: 5,
            song: SONGS[2].id,
            type_: MEDIA_TYPES[1].name.clone(),
            name: "TTBB MIDI's".to_owned(),
            target: "TTBB%20MIDIs.midi".to_owned(),
        },
    ];
    pub static ref GIG_SONGS: [GigSong; 2] = [
        GigSong {
            event: EVENTS[3].id,
            song: SONGS[1].id,
            order: 1,
        },
        GigSong {
            event: EVENTS[3].id,
            song: SONGS[0].id,
            order: 2,
        },
    ];
    pub static ref MEDIA_TYPES: [MediaType; 4] = [
        MediaType {
            name: "Links".to_owned(),
            order: 3,
            storage: StorageType::Remote,
        },
        MediaType {
            name: "MIDIs".to_owned(),
            order: 1,
            storage: StorageType::Local,
        },
        MediaType {
            name: "Sheet Music".to_owned(),
            order: 0,
            storage: StorageType::Local,
        },
        MediaType {
            name: "Performances".to_owned(),
            order: 2,
            storage: StorageType::Remote,
        },
    ];
    pub static ref MEETING_MINUTES: [MeetingMinutes; 3] = [
        MeetingMinutes {
            id: 1,
            name: "Officer's Meeting - 02/10".to_owned(),
            date: NaiveDate::parse_from_str("2016-02-10", "%r").unwrap(),
            private: None,
            public: Some("Didn't discuss much. Make sure to have dues \
                          collected by next week!".to_owned()),
        },
        MeetingMinutes {
            id: 2,
            name: "Officer's Meeting - 02/17".to_owned(),
            date: NaiveDate::parse_from_str("2016-02-17", "%r").unwrap(),
            private: Some("We actually discussed the possible increase of \
                           dues, and will discuss further next week.".to_owned()),
            public: Some("Nothing to report.".to_owned()),
        },
        MeetingMinutes {
            id: 3,
            name: "Officer's Meeting - 02/24".to_owned(),
            date: NaiveDate::parse_from_str("2016-02-24", "%r").unwrap(),
            private: None,
            public: Some("We discussed increasing dues last week, \
                          but in the end decided not to as we can generate \
                          the revenue needed from an increase in the \
                          number of gigs we take on each semester.".to_owned()),
        },
    ];
    pub static ref TODOS: [Todo; 5] = [
        Todo {
            id: 1,
            text: "Check for flights for the upcoming trip!".to_owned(),
            member: MEMBERS[0].email.clone(),
            completed: false,
        },
        Todo {
            id: 2,
            text: "Check for flights for the upcoming trip!".to_owned(),
            member: MEMBERS[1].email.clone(),
            completed: true,
        },
        Todo {
            id: 3,
            text: "Look into new ways to recruit members".to_owned(),
            member: MEMBERS[1].email.clone(),
            completed: false,
        },
        Todo {
            id: 4,
            text: "Look into new ways to recruit members".to_owned(),
            member: MEMBERS[2].email.clone(),
            completed: false,
        },
        Todo {
            id: 5,
            text: "Ask Doc to upload the MIDI's for Ramble".to_owned(),
            member: MEMBERS[2].email.clone(),
            completed: false,
        },
    ];
    pub static ref TRANSACTION_TYPES: [TransactionType; 6]= [
        TransactionType {
            name: "Deposit".to_owned(),
        },
        TransactionType {
            name: "Dues".to_owned(),
        },
        TransactionType {
            name: "Expense".to_owned(),
        },
        TransactionType {
            name: "Other".to_owned(),
        },
        TransactionType {
            name: "Purchase".to_owned(),
        },
        TransactionType {
            name: "Trip".to_owned(),
        },
    ];
    pub static ref TRANSACTIONS: [Transaction; 4] = [
        Transaction {
            id: 1,
            member: MEMBERS[0].email.clone(),
            time: NaiveDateTime::parse_from_str("12:06:12 PM 2016-02-11", "%r %F").unwrap(),
            amount: FEES[1].amount,
            description: FEES[1].description.clone(),
            semester: Some(SEMESTERS[0].name.clone()),
            type_: TRANSACTION_TYPES[1].name.clone(),
            resolved: true,
        },
        Transaction {
            id: 2,
            member: MEMBERS[0].email.clone(),
            time: NaiveDateTime::parse_from_str("01:25:02 PM 2016-02-15", "%r %F").unwrap(),
            amount: FEES[0].amount,
            description: FEES[0].description.clone(),
            semester: Some(SEMESTERS[0].name.clone()),
            type_: TRANSACTION_TYPES[1].name.clone(),
            resolved: true,
        },
        Transaction {
            id: 3,
            member: MEMBERS[1].email.clone(),
            time: NaiveDateTime::parse_from_str("04:36:57 PM 2016-03-15", "%r %F").unwrap(),
            amount: 100,
            description: "Deposit for the trip!".to_owned(),
            semester: Some(SEMESTERS[0].name.clone()),
            type_: TRANSACTION_TYPES[0].name.clone(),
            resolved: true,
        },
        Transaction {
            id: 4,
            member: MEMBERS[1].email.clone(),
            time: NaiveDateTime::parse_from_str("10:03:41 AM 2016-03-20", "%r %F").unwrap(),
            amount: 250,
            description: "Plane flights for the trip.".to_owned(),
            semester: Some(SEMESTERS[0].name.clone()),
            type_: TRANSACTION_TYPES[5].name.clone(),
            resolved: false,
        },
    ];
    pub static ref SESSIONS: [Session; 1] = [
        Session {
            member: MEMBERS[0].email.clone(),
            key: "deadbeef".to_owned(),
        },
    ];
    pub static ref VARIABLES: [Variable; 2] = [
        Variable {
            key: "admin_list".to_owned(),
            value: "officers@li.st".to_owned(),
        },
        Variable {
            key: "default_color".to_owned(),
            value: "#000".to_owned(),
        },
    ];
    pub static ref PERMISSIONS: [Permission; 34] = [
        Permission {
            description: Some("Add todos for multiple people".to_owned()),
            name: "add-multi-todo".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Create new events".to_owned()),
            name: "create-event".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("Delete events".to_owned()),
            name: "delete-event".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("Delete users from the database".to_owned()),
            name: "delete-user".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Edit any event in any way".to_owned()),
            name: "edit-all-events".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Make new officer announcements".to_owned()),
            name: "edit-announcements".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Edit attendance for all events and sections".to_owned()),
            name: "edit-attendance".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("Edit attendance for events in the user's own section".to_owned()),
            name: "edit-attendance-own-section".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("Create and modify carpools".to_owned()),
            name: "edit-carpool".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Modify grading parameters like gig requirement".to_owned()),
            name: "edit-grading".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Edit document links displayed in the \"documents\" dropdown".to_owned()),
            name: "edit-links".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Create, modify, and delete officer meeting minutes".to_owned()),
            name: "edit-minutes".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Change officer positions".to_owned()),
            name: "edit-officers".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Grant and revoke permissions to officers".to_owned()),
            name: "edit-permissions".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Create, modify, and delete repertoire".to_owned()),
            name: "edit-repertoire".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Change the current semester".to_owned()),
            name: "edit-semester".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Modify the set list for events".to_owned()),
            name: "edit-setlist".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("Create and modify tie transactions".to_owned()),
            name: "edit-tie".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Create and modify transactions".to_owned()),
            name: "edit-transaction".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Add, delete, and modify uniform descriptions.".to_owned()),
            name: "edit-uniforms".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Modify the details of all users".to_owned()),
            name: "edit-user".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Modify events".to_owned()),
            name: "modify-event".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("View and respond to member absence requests.".to_owned()),
            name: "process-absence-requests".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View, accept, and dismiss gig requests made through the external site".to_owned()),
            name: "process-gig-requests".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("Log in as a different user without authenticating".to_owned()),
            name: "switch-user".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View all events, even events with which the member does not have an attendance relation".to_owned()),
            name: "view-all-events".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View attendance for all events and sections".to_owned()),
            name: "view-attendance".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("View attendance for events in the user's own section".to_owned()),
            name: "view-attendance-own-section".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("View unredacted officer meeting minutes".to_owned()),
            name: "view-complete-minutes".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View private event details, such as requester contact information and price".to_owned()),
            name: "view-event-private-details".to_owned(),
            type_: PermissionType::Event,
        },
        Permission {
            description: Some("View tie transactions".to_owned()),
            name: "view-ties".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View monetary transaction records".to_owned()),
            name: "view-transactions".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View private details of users, such as food restrictions and score".to_owned()),
            name: "view-user-private-details".to_owned(),
            type_: PermissionType::Static,
        },
        Permission {
            description: Some("View all users".to_owned()),
            name: "view-users".to_owned(),
            type_: PermissionType::Static,
        }
    ];
    pub static ref ROLES: [Role; 14] = [
        Role {
            name: "Member".to_owned(),
            max_quantity: -1,
            rank: 99,
        },
        Role {
            name: "Section Leader".to_owned(),
            max_quantity: 4,
            rank: 9,
        },
        Role {
            name: "Ombudsman".to_owned(),
            max_quantity: 1,
            rank: 8,
        },
        Role {
            name: "Manager".to_owned(),
            max_quantity: 2,
            rank: 3,
        },
        Role {
            name: "Treasurer".to_owned(),
            max_quantity: 1,
            rank: 4,
        },
        Role {
            name: "Internal VP".to_owned(),
            max_quantity: 0,
            rank: -1,
        },
        Role {
            name: "External VP".to_owned(),
            max_quantity: 0,
            rank: -1,
        },
        Role {
            name: "President".to_owned(),
            max_quantity: 1,
            rank: 1,
        },
        Role {
            name: "Vice President".to_owned(),
            max_quantity: 1,
            rank: 2,
        },
        Role {
            name: "Liaison".to_owned(),
            max_quantity: 1,
            rank: 5,
        },
        Role {
            name: "Advocate".to_owned(),
            max_quantity: 1,
            rank: 6,
        },
        Role {
            name: "Webmaster".to_owned(),
            max_quantity: 4,
            rank: 7,
        },
        Role {
            name: "Instructor".to_owned(),
            max_quantity: 2,
            rank: -1,
        },
        Role {
            name: "Any".to_owned(),
            max_quantity: -1,
            rank: -1,
        },
    ];
    pub static ref MEMBER_ROLES: [MemberRole; 3] = [
        MemberRole {
            member: MEMBERS[0].email.clone(),
            role: ROLES[11].name.clone(),
        },
        MemberRole {
            member: MEMBERS[1].email.clone(),
            role: ROLES[7].name.clone(),
        },
        MemberRole {
            member: MEMBERS[2].email.clone(),
            role: ROLES[2].name.clone(),
        },
    ];
    pub static ref ROLE_PERMISSIONS: [RolePermission; 169] = [
        RolePermission {
            id: 6,
            role: ROLES[7].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 7,
            role: ROLES[7].name.clone(),
            permission: "create-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 8,
            role: ROLES[7].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 9,
            role: ROLES[7].name.clone(),
            permission: "delete-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 10,
            role: ROLES[7].name.clone(),
            permission: "edit-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 11,
            role: ROLES[7].name.clone(),
            permission: "edit-announcements".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 12,
            role: ROLES[7].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 14,
            role: ROLES[7].name.clone(),
            permission: "edit-carpool".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 15,
            role: ROLES[7].name.clone(),
            permission: "edit-grading".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 16,
            role: ROLES[7].name.clone(),
            permission: "edit-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 17,
            role: ROLES[7].name.clone(),
            permission: "edit-officers".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 18,
            role: ROLES[7].name.clone(),
            permission: "edit-permissions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 19,
            role: ROLES[7].name.clone(),
            permission: "edit-repertoire".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 20,
            role: ROLES[7].name.clone(),
            permission: "edit-semester".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 21,
            role: ROLES[7].name.clone(),
            permission: "edit-setlist".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 22,
            role: ROLES[7].name.clone(),
            permission: "edit-tie".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 23,
            role: ROLES[7].name.clone(),
            permission: "edit-transaction".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 24,
            role: ROLES[7].name.clone(),
            permission: "edit-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 25,
            role: ROLES[7].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 27,
            role: ROLES[7].name.clone(),
            permission: "switch-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 29,
            role: ROLES[7].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 30,
            role: ROLES[7].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 32,
            role: ROLES[7].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 33,
            role: ROLES[7].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 34,
            role: ROLES[7].name.clone(),
            permission: "view-ties".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 35,
            role: ROLES[7].name.clone(),
            permission: "view-transactions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 36,
            role: ROLES[7].name.clone(),
            permission: "view-user-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 37,
            role: ROLES[7].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 38,
            role: ROLES[8].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 39,
            role: ROLES[8].name.clone(),
            permission: "create-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 40,
            role: ROLES[8].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 41,
            role: ROLES[8].name.clone(),
            permission: "edit-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 42,
            role: ROLES[8].name.clone(),
            permission: "edit-announcements".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 43,
            role: ROLES[8].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 44,
            role: ROLES[8].name.clone(),
            permission: "edit-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 45,
            role: ROLES[8].name.clone(),
            permission: "edit-repertoire".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 46,
            role: ROLES[8].name.clone(),
            permission: "edit-setlist".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 47,
            role: ROLES[8].name.clone(),
            permission: "edit-tie".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 48,
            role: ROLES[8].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 50,
            role: ROLES[8].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 51,
            role: ROLES[8].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 52,
            role: ROLES[8].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 53,
            role: ROLES[8].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 54,
            role: ROLES[8].name.clone(),
            permission: "view-ties".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 55,
            role: ROLES[8].name.clone(),
            permission: "view-user-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 56,
            role: ROLES[8].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 57,
            role: ROLES[3].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 58,
            role: ROLES[4].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 59,
            role: ROLES[9].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 60,
            role: ROLES[10].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 65,
            role: ROLES[3].name.clone(),
            permission: "edit-carpool".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 67,
            role: ROLES[3].name.clone(),
            permission: "edit-setlist".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 68,
            role: ROLES[4].name.clone(),
            permission: "edit-tie".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 69,
            role: ROLES[4].name.clone(),
            permission: "edit-transaction".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 75,
            role: ROLES[3].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 76,
            role: ROLES[4].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 77,
            role: ROLES[3].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 78,
            role: ROLES[4].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 81,
            role: ROLES[4].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 82,
            role: ROLES[3].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 83,
            role: ROLES[4].name.clone(),
            permission: "view-ties".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 84,
            role: ROLES[4].name.clone(),
            permission: "view-transactions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 85,
            role: ROLES[3].name.clone(),
            permission: "view-user-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 86,
            role: ROLES[3].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 87,
            role: ROLES[11].name.clone(),
            permission: "add-multi-todo".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 88,
            role: ROLES[11].name.clone(),
            permission: "create-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 89,
            role: ROLES[11].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 90,
            role: ROLES[11].name.clone(),
            permission: "delete-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 91,
            role: ROLES[11].name.clone(),
            permission: "edit-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 92,
            role: ROLES[11].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 93,
            role: ROLES[11].name.clone(),
            permission: "edit-carpool".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 94,
            role: ROLES[11].name.clone(),
            permission: "edit-grading".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 95,
            role: ROLES[11].name.clone(),
            permission: "edit-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 96,
            role: ROLES[11].name.clone(),
            permission: "edit-officers".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 97,
            role: ROLES[11].name.clone(),
            permission: "edit-permissions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 98,
            role: ROLES[11].name.clone(),
            permission: "edit-repertoire".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 99,
            role: ROLES[11].name.clone(),
            permission: "edit-semester".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 100,
            role: ROLES[11].name.clone(),
            permission: "edit-tie".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 101,
            role: ROLES[11].name.clone(),
            permission: "edit-setlist".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 102,
            role: ROLES[11].name.clone(),
            permission: "edit-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 103,
            role: ROLES[11].name.clone(),
            permission: "edit-transaction".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 104,
            role: ROLES[11].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 106,
            role: ROLES[11].name.clone(),
            permission: "switch-user".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 108,
            role: ROLES[11].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 109,
            role: ROLES[11].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 110,
            role: ROLES[11].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 111,
            role: ROLES[11].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 112,
            role: ROLES[11].name.clone(),
            permission: "view-ties".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 113,
            role: ROLES[11].name.clone(),
            permission: "view-transactions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 114,
            role: ROLES[11].name.clone(),
            permission: "view-user-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 115,
            role: ROLES[11].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 136,
            role: ROLES[1].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 137,
            role: ROLES[1].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 143,
            role: ROLES[3].name.clone(),
            permission: "edit-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 144,
            role: ROLES[3].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 147,
            role: ROLES[3].name.clone(),
            permission: "create-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 148,
            role: ROLES[3].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 149,
            role: ROLES[7].name.clone(),
            permission: "process-absence-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 150,
            role: ROLES[8].name.clone(),
            permission: "process-absence-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 151,
            role: ROLES[11].name.clone(),
            permission: "process-absence-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 152,
            role: ROLES[7].name.clone(),
            permission: "process-gig-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 153,
            role: ROLES[8].name.clone(),
            permission: "process-gig-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 154,
            role: ROLES[11].name.clone(),
            permission: "process-gig-requests".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 156,
            role: ROLES[3].name.clone(),
            permission: "edit-repertoire".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 166,
            role: ROLES[3].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 167,
            role: ROLES[2].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 169,
            role: ROLES[2].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("other".to_owned()),
        },
        RolePermission {
            id: 170,
            role: ROLES[2].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 171,
            role: ROLES[2].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 172,
            role: ROLES[8].name.clone(),
            permission: "edit-grading".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 173,
            role: ROLES[8].name.clone(),
            permission: "edit-carpool".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 174,
            role: ROLES[9].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 175,
            role: ROLES[10].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 176,
            role: ROLES[2].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 177,
            role: ROLES[1].name.clone(),
            permission: "view-complete-minutes".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 179,
            role: ROLES[9].name.clone(),
            permission: "edit-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 180,
            role: ROLES[3].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 181,
            role: ROLES[4].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 182,
            role: ROLES[9].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 183,
            role: ROLES[10].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 184,
            role: ROLES[2].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 185,
            role: ROLES[1].name.clone(),
            permission: "edit-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 186,
            role: ROLES[9].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 187,
            role: ROLES[10].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 188,
            role: ROLES[2].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 189,
            role: ROLES[1].name.clone(),
            permission: "view-all-events".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 190,
            role: ROLES[1].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 191,
            role: ROLES[2].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 192,
            role: ROLES[10].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 193,
            role: ROLES[9].name.clone(),
            permission: "view-attendance".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 194,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 195,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("rehearsal".to_owned()),
        },
        RolePermission {
            id: 196,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 197,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("tutti".to_owned()),
        },
        RolePermission {
            id: 198,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("volunteer".to_owned()),
        },
        RolePermission {
            id: 199,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 200,
            role: ROLES[9].name.clone(),
            permission: "create-event".to_owned(),
            event_type: Some("other".to_owned()),
        },
        RolePermission {
            id: 201,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 202,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("rehearsal".to_owned()),
        },
        RolePermission {
            id: 203,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 204,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("tutti".to_owned()),
        },
        RolePermission {
            id: 205,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("volunteer".to_owned()),
        },
        RolePermission {
            id: 206,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 207,
            role: ROLES[9].name.clone(),
            permission: "delete-event".to_owned(),
            event_type: Some("other".to_owned()),
        },
        RolePermission {
            id: 208,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 209,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("rehearsal".to_owned()),
        },
        RolePermission {
            id: 210,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 211,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("tutti".to_owned()),
        },
        RolePermission {
            id: 212,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("volunteer".to_owned()),
        },
        RolePermission {
            id: 213,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 214,
            role: ROLES[9].name.clone(),
            permission: "modify-event".to_owned(),
            event_type: Some("other".to_owned()),
        },
        RolePermission {
            id: 215,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 216,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("rehearsal".to_owned()),
        },
        RolePermission {
            id: 217,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("sectional".to_owned()),
        },
        RolePermission {
            id: 218,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("tutti".to_owned()),
        },
        RolePermission {
            id: 219,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("volunteer".to_owned()),
        },
        RolePermission {
            id: 220,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("ombuds".to_owned()),
        },
        RolePermission {
            id: 221,
            role: ROLES[9].name.clone(),
            permission: "view-event-private-details".to_owned(),
            event_type: Some("other".to_owned()),
        },
        RolePermission {
            id: 223,
            role: ROLES[8].name.clone(),
            permission: "view-transactions".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 224,
            role: ROLES[7].name.clone(),
            permission: "edit-uniforms".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 225,
            role: ROLES[0].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 226,
            role: ROLES[4].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 227,
            role: ROLES[9].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 228,
            role: ROLES[10].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 229,
            role: ROLES[2].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 230,
            role: ROLES[1].name.clone(),
            permission: "view-users".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 231,
            role: ROLES[4].name.clone(),
            permission: "view-user-private-details".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 232,
            role: ROLES[7].name.clone(),
            permission: "edit-links".to_owned(),
            event_type: None,
        },
        RolePermission {
            id: 233,
            role: ROLES[8].name.clone(),
            permission: "edit-links".to_owned(),
            event_type: None,
        }
    ];
}
