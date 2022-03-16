#![allow(non_snake_case)]

use chrono::{NaiveDate, NaiveDateTime};
use mysql::Pool;

use crate::error::{MigrateError, MigrateResult};
use crate::migrate::Load;

macro_rules! impl_load {
    ($table_name:expr, pub struct $type_name:ident { $(pub $field_names:ident: $field_types:ty,)* }) => {
        #[derive(Clone, Debug)]
        pub struct $type_name {
            $(
                pub $field_names: $field_types,
            )*
        }

        impl Load for $type_name {
            fn load(old_db: &Pool) -> MigrateResult<Vec<Self>> {
                let str_field_names = vec![ $( stringify!($field_names), )* ];
                let query = format!("SELECT {} from {}", str_field_names
                    .into_iter()
                    .map(|field| if field == "type_" { "`type`".to_owned() } else { format!("`{}`", field) })
                    .collect::<Vec<String>>()
                    .join(", "), $table_name);

                old_db.prep_exec(query, ())
                    .map_err(MigrateError::MySqlError)
                    .and_then(|result| {
                        result.map(|row_result| row_result
                            .map_err(MigrateError::MySqlError)
                            .and_then(|mut row| {
                                impl_load!(
                                    0, row, $table_name, $type_name,
                                    [ $($field_names,)* ], $( $field_names )*
                                )
                            })
                        ).collect::<Result<Vec<Self>, MigrateError>>()
                    })
            }
        }
    };
    ($index:expr, $row:expr, $table_name:expr, $type_name:ident,
            [ $field_name:ident, $( $field_names:ident, )* ], $( $all_fields:ident )* ) => {
        {
            let $field_name = $row.take_opt($index).ok_or(MigrateError::OutOfBoundsRowIndex {
                table: $table_name,
                row_index: $index,
            })?.map_err(MigrateError::FromValueError)?;
            impl_load!($index + 1, $row, $table_name, $type_name, [ $( $field_names, )* ], $( $all_fields )* )
        }
    };
    ($index:expr, $row:expr, $table_name:expr, $type_name:ident, [ ], $( $all_fields:ident )* ) => {
        Ok($type_name {
            $( $all_fields, )*
        })
    };
}

// CREATE TABLE `absencerequest` (
//   `eventNo` int(11) NOT NULL,
//   `memberID` varchar(50) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
//   `reason` varchar(500) NOT NULL,
//   `replacement` varchar(20) DEFAULT NULL,
//   `state` varchar(20) NOT NULL DEFAULT 'pending'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `absencerequest`
//   ADD PRIMARY KEY (`memberID`,`eventNo`),
//   ADD KEY `eventNo` (`eventNo`),
//   ADD KEY `state` (`state`);

// ALTER TABLE `absencerequest`
//   ADD CONSTRAINT `absencerequest_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `absencerequest_ibfk_2` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `absencerequest_ibfk_3` FOREIGN KEY (`state`) REFERENCES `requestState` (`stateName`);
impl_load! {
    "absencerequest",
    pub struct OldAbsenceRequest {
        pub eventNo: i64,
        pub memberID: String,
        pub time: NaiveDateTime,
        pub reason: String,
        pub replacement: Option<String>,
        pub state: String,
    }
}

// CREATE TABLE `activeSemester` (
//   `member` varchar(50) NOT NULL,
//   `semester` varchar(16) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `enrollment` enum('class','club') NOT NULL DEFAULT 'club',
//   `section` int(1) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `activeSemester`
//   ADD PRIMARY KEY (`member`,`semester`,`choir`),
//   ADD KEY `member` (`member`),
//   ADD KEY `semester` (`semester`),
//   ADD KEY `choir` (`choir`),
//   ADD KEY `section` (`section`);

// ALTER TABLE `activeSemester`
//   ADD CONSTRAINT `activeSemester_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `activeSemester_ibfk_2` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `activeSemester_ibfk_3` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`),
//   ADD CONSTRAINT `activeSemester_ibfk_4` FOREIGN KEY (`section`) REFERENCES `sectionType` (`id`) ON UPDATE CASCADE;
impl_load! {
    "activeSemester",
    pub struct OldActiveSemester {
        pub member: String,
        pub semester: String,
        pub choir: String,
        pub enrollment: String,
        pub section: i64,
    }
}

// CREATE TABLE `announcement` (
//   `announcementNo` int(11) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `memberID` varchar(50) NOT NULL,
//   `timePosted` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   `announcement` longtext NOT NULL,
//   `archived` tinyint(1) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `announcement`
//   ADD PRIMARY KEY (`announcementNo`),
//   ADD KEY `memberID` (`memberID`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `announcement`
//   ADD CONSTRAINT `announcement_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `announcement_ibfk_2` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "announcement",
    pub struct OldAnnouncement {
        pub announcementNo: i64,
        pub choir: String,
        pub memberID: String,
        pub timePosted: NaiveDateTime,
        pub announcement: String,
        pub archived: bool,
    }
}

// CREATE TABLE `attends` (
//   `memberID` varchar(50) NOT NULL,
//   `shouldAttend` tinyint(1) NOT NULL DEFAULT '1',
//   `didAttend` tinyint(1) DEFAULT NULL,
//   `eventNo` int(11) NOT NULL DEFAULT '0',
//   `minutesLate` int(11) NOT NULL DEFAULT '0',
//   `confirmed` tinyint(1) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `attends`
//   ADD PRIMARY KEY (`memberID`,`eventNo`),
//   ADD KEY `eventNo` (`eventNo`);

// ALTER TABLE `attends`
//   ADD CONSTRAINT `attends_ibfk_2` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `attends_ibfk_3` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "attends",
    pub struct OldAttends {
        pub memberID: String,
        pub shouldAttend: bool,
        pub didAttend: Option<bool>,
        pub eventNo: i64,
        pub minutesLate: i64,
        pub confirmed: bool,
    }
}

// CREATE TABLE `carpool` (
//   `carpoolID` int(11) NOT NULL,
//   `driver` varchar(50) NOT NULL,
//   `eventNo` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `carpool`
//   ADD PRIMARY KEY (`carpoolID`),
//   ADD KEY `eventNo` (`eventNo`),
//   ADD KEY `driver_memberID` (`driver`);

// ALTER TABLE `carpool`
//   ADD CONSTRAINT `carpool_ibfk_1` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `carpool_ibfk_2` FOREIGN KEY (`driver`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "carpool",
    pub struct OldCarpool {
        pub carpoolID: i64,
        pub driver: String,
        pub eventNo: i64,
    }
}

// CREATE TABLE `choir` (
//   `id` varchar(16) NOT NULL,
//   `name` varchar(64) NOT NULL,
//   `admin` varchar(128) NOT NULL,
//   `list` varchar(128) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `choir`
//   ADD PRIMARY KEY (`id`);

// ALTER TABLE `event`
//   ADD CONSTRAINT `event_ibfk_3` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON UPDATE CASCADE,
//   ADD CONSTRAINT `event_ibfk_4` FOREIGN KEY (`type`) REFERENCES `eventType` (`id`),
//   ADD CONSTRAINT `event_ibfk_5` FOREIGN KEY (`section`) REFERENCES `sectionType` (`id`) ON UPDATE CASCADE,
//   ADD CONSTRAINT `event_validSemester` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "choir",
    pub struct OldChoir {
        pub id: String,
        pub name: String,
        pub admin: String,
        pub list: String,
    }
}

// CREATE TABLE `event` (
//   `eventNo` int(11) NOT NULL,
//   `name` varchar(50) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `callTime` datetime NOT NULL,
//   `releaseTime` datetime DEFAULT NULL,
//   `points` int(2) NOT NULL,
//   `comments` varchar(1500) DEFAULT NULL,
//   `type` varchar(16) NOT NULL,
//   `location` varchar(500) DEFAULT NULL,
//   `semester` varchar(16) NOT NULL,
//   `gigcount` tinyint(1) NOT NULL DEFAULT '1',
//   `section` int(1) NOT NULL DEFAULT '0',
//   `defaultAttend` tinyint(1) NOT NULL DEFAULT '1'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `event`
//   ADD PRIMARY KEY (`eventNo`),
//   ADD KEY `event_validSemester` (`semester`),
//   ADD KEY `section` (`section`),
//   ADD KEY `choir` (`choir`),
//   ADD KEY `type_2` (`type`);
impl_load! {
    "event",
    pub struct OldEvent {
        pub eventNo: i64,
        pub name: String,
        pub choir: String,
        pub callTime: NaiveDateTime,
        pub releaseTime: Option<NaiveDateTime>,
        pub points: i64,
        pub comments: Option<String>,
        pub type_: String,
        pub location: Option<String>,
        pub semester: String,
        pub gigcount: bool,
        pub section: i64,
        pub defaultAttend: bool,
    }
}

// CREATE TABLE `eventType` (
//   `id` varchar(16) NOT NULL,
//   `name` varchar(64) NOT NULL,
//   `weight` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `eventType`
//   ADD PRIMARY KEY (`id`),
//   ADD UNIQUE KEY `id` (`id`);
impl_load! {
    "eventType",
    pub struct OldEventType {
        pub id: String,
        pub name: String,
        pub weight: i64,
    }
}

// CREATE TABLE `fee` (
//   `id` varchar(16) NOT NULL,
//   `name` varchar(40) DEFAULT NULL,
//   `choir` varchar(16) NOT NULL,
//   `amount` int(11) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `fee`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `fee`
//   ADD CONSTRAINT `fee_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "fee",
    pub struct OldFee {
        pub id: String,
        pub name: Option<String>,
        pub choir: String,
        pub amount: i64,
    }
}

// CREATE TABLE `gdocs` (
//   `name` varchar(40) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `url` varchar(128) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `gdocs`
//   ADD PRIMARY KEY (`name`,`choir`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `gdocs`
//   ADD CONSTRAINT `gdocs_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "gdocs",
    pub struct OldGDocs {
        pub name: String,
        pub choir: String,
        pub url: String,
    }
}

// CREATE TABLE `gig` (
//   `eventNo` int(11) NOT NULL,
//   `performanceTime` datetime NOT NULL,
//   `uniform` varchar(13) NOT NULL,
//   `cname` varchar(20) DEFAULT NULL,
//   `cemail` varchar(50) DEFAULT NULL,
//   `cphone` varchar(16) DEFAULT NULL,
//   `price` int(4) DEFAULT NULL,
//   `public` tinyint(1) NOT NULL DEFAULT '0',
//   `summary` text NOT NULL,
//   `description` text NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `gig`
//   ADD PRIMARY KEY (`eventNo`),
//   ADD KEY `uniform` (`uniform`);

// ALTER TABLE `gig`
//   ADD CONSTRAINT `gig_ibfk_1` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `gig_ibfk_2` FOREIGN KEY (`uniform`) REFERENCES `uniform` (`id`);
impl_load! {
    "gig",
    pub struct OldGig {
        pub eventNo: i64,
        pub performanceTime: NaiveDateTime,
        pub uniform: String,
        pub cname: Option<String>,
        pub cemail: Option<String>,
        pub cphone: Option<String>,
        pub price: Option<i64>,
        pub public: bool,
        pub summary: String,
        pub description: String,
    }
}

// CREATE TABLE `gigreq` (
//   `id` int(11) NOT NULL,
//   `timestamp` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   `name` varchar(256) NOT NULL,
//   `org` varchar(256) NOT NULL,
//   `eventNo` int(11) DEFAULT NULL,
//   `cname` varchar(256) NOT NULL,
//   `cphone` varchar(10) NOT NULL,
//   `cemail` varchar(256) NOT NULL,
//   `startTime` datetime NOT NULL,
//   `location` varchar(512) NOT NULL,
//   `comments` text NOT NULL,
//   `status` enum('pending','accepted','dismissed') NOT NULL DEFAULT 'pending',
//   `semester` varchar(16) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `gigreq`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `semester` (`semester`),
//   ADD KEY `eventNo` (`eventNo`);

// ALTER TABLE `gigreq`
//   ADD CONSTRAINT `gigreq_ibfk_1` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`),
//   ADD CONSTRAINT `gigreq_ibfk_2` FOREIGN KEY (`eventNo`) REFERENCES `event` (`eventNo`) ON DELETE SET NULL ON UPDATE CASCADE;
impl_load! {
    "gigreq",
    pub struct OldGigReq {
        pub id: i64,
        pub timestamp: NaiveDateTime,
        pub name: String,
        pub org: String,
        pub eventNo: Option<i64>,
        pub cname: String,
        pub cphone: String,
        pub cemail: String,
        pub startTime: NaiveDateTime,
        pub location: String,
        pub comments: String,
        pub status: String,
        pub semester: String,
    }
}

// CREATE TABLE `gigSong` (
//   `id` int(11) NOT NULL,
//   `event` int(11) NOT NULL,
//   `song` int(11) NOT NULL,
//   `order` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=latin1;

// ALTER TABLE `gigSong`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `event` (`event`),
//   ADD KEY `song` (`song`);

// ALTER TABLE `gigSong`
//   ADD CONSTRAINT `gigSong_ibfk_1` FOREIGN KEY (`event`) REFERENCES `event` (`eventNo`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `gigSong_ibfk_2` FOREIGN KEY (`song`) REFERENCES `song` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "gigSong",
    pub struct OldGigSong {
        pub id: i64,
        pub event: i64,
        pub song: i64,
        pub order: i64,
    }
}

// CREATE TABLE `mediaType` (
//   `typeid` varchar(10) NOT NULL,
//   `order` smallint(6) NOT NULL,
//   `name` varchar(128) NOT NULL,
//   `storage` enum('local','remote') NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `mediaType`
//   ADD PRIMARY KEY (`typeid`);
impl_load! {
    "mediaType",
    pub struct OldMediaType {
        pub typeid: String,
        pub order: i64,
        pub name: String,
        pub storage: String,
    }
}

// CREATE TABLE `member` (
//   `firstName` varchar(20) DEFAULT NULL,
//   `prefName` varchar(20) DEFAULT NULL,
//   `lastName` varchar(20) DEFAULT NULL,
//   `email` varchar(50) NOT NULL DEFAULT '',
//   `password` varchar(50) DEFAULT NULL,
//   `phone` bigint(10) DEFAULT NULL,
//   `picture` varchar(100) DEFAULT NULL,
//   `passengers` int(3) NOT NULL DEFAULT '0',
//   `onCampus` tinyint(1) DEFAULT NULL,
//   `location` varchar(50) DEFAULT NULL,
//   `about` varchar(500) DEFAULT NULL,
//   `major` varchar(50) DEFAULT NULL,
//   `minor` varchar(50) DEFAULT NULL,
//   `hometown` varchar(50) DEFAULT NULL,
//   `techYear` int(1) DEFAULT NULL,
//   `gChat` varchar(20) DEFAULT NULL,
//   `twitter` varchar(20) DEFAULT NULL,
//   `gatewayDrug` varchar(500) DEFAULT NULL,
//   `conflicts` varchar(500) DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `member`
//   ADD PRIMARY KEY (`email`);
impl_load! {
    "member",
    pub struct OldMember {
        pub firstName: Option<String>,
        pub prefName: Option<String>,
        pub lastName: Option<String>,
        pub email: String,
        pub password: Option<String>,
        pub phone: Option<i64>,
        pub picture: Option<String>,
        pub passengers: i64,
        pub onCampus: Option<bool>,
        pub location: Option<String>,
        pub about: Option<String>,
        pub major: Option<String>,
        pub minor: Option<String>,
        pub hometown: Option<String>,
        pub techYear: Option<i64>,
        pub gChat: Option<String>,
        pub twitter: Option<String>,
        pub gatewayDrug: Option<String>,
        pub conflicts: Option<String>,
    }
}

// CREATE TABLE `memberRole` (
//   `member` varchar(50) NOT NULL,
//   `role` int(11) NOT NULL,
//   `semester` varchar(16) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `memberRole`
//   ADD PRIMARY KEY (`member`,`role`,`semester`),
//   ADD KEY `member` (`member`,`role`,`semester`),
//   ADD KEY `semester` (`semester`),
//   ADD KEY `role` (`role`);

// ALTER TABLE `memberRole`
//   ADD CONSTRAINT `memberRole_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `memberRole_ibfk_3` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `memberRole_ibfk_4` FOREIGN KEY (`role`) REFERENCES `role` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "memberRole",
    pub struct OldMemberRole {
        pub member: String,
        pub role: i64,
        pub semester: String,
    }
}

// CREATE TABLE `minutes` (
//   `id` int(11) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `date` date NOT NULL,
//   `name` varchar(100) NOT NULL,
//   `private` longtext,
//   `public` longtext
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `minutes`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `minutes`
//   ADD CONSTRAINT `minutes_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);
impl_load! {
    "minutes",
    pub struct OldMinutes {
        pub id: i64,
        pub choir: String,
        pub date: NaiveDate,
        pub name: String,
        pub private: Option<String>,
        pub public: Option<String>,
    }
}

// CREATE TABLE `permission` (
//   `name` varchar(40) NOT NULL,
//   `description` text,
//   `type` enum('static','event') NOT NULL DEFAULT 'static'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `permission`
//   ADD PRIMARY KEY (`name`);
impl_load! {
    "permission",
    pub struct OldPermission {
        pub name: String,
        pub description: Option<String>,
        pub type_: String,
    }
}

// CREATE TABLE `requestState` (
//   `stateName` varchar(20) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `requestState`
//   ADD PRIMARY KEY (`stateName`);
impl_load! {
    "requestState",
    pub struct OldRequestState {
        pub stateName: String,
    }
}

// CREATE TABLE `ridesin` (
//   `memberID` varchar(50) NOT NULL,
//   `carpoolID` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `ridesin`
//   ADD PRIMARY KEY (`memberID`,`carpoolID`),
//   ADD KEY `carpoolID` (`carpoolID`);

// ALTER TABLE `ridesin`
//   ADD CONSTRAINT `ridesin_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `ridesin_ibfk_2` FOREIGN KEY (`carpoolID`) REFERENCES `carpool` (`carpoolID`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "ridesin",
    pub struct OldRidesIn {
        pub memberID: String,
        pub carpoolID: i64,
    }
}

// CREATE TABLE `role` (
//   `id` int(1) NOT NULL,
//   `name` varchar(20) DEFAULT NULL,
//   `choir` varchar(16) NOT NULL,
//   `rank` int(11) NOT NULL,
//   `quantity` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `role`
//   ADD PRIMARY KEY (`id`),
//   ADD UNIQUE KEY `typeName` (`name`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `role`
//   ADD CONSTRAINT `role_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "role",
    pub struct OldRole {
        pub id: i64,
        pub name: Option<String>,
        pub choir: String,
        pub rank: i64,
        pub quantity: i64,
    }
}

// CREATE TABLE `rolePermission` (
//   `id` int(11) NOT NULL,
//   `role` int(1) NOT NULL,
//   `permission` varchar(40) NOT NULL,
//   `eventType` varchar(16) DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `rolePermission`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `role` (`role`),
//   ADD KEY `permission` (`permission`),
//   ADD KEY `eventType` (`eventType`);

// ALTER TABLE `rolePermission`
//   ADD CONSTRAINT `rolePermission_ibfk_1` FOREIGN KEY (`role`) REFERENCES `role` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `rolePermission_ibfk_2` FOREIGN KEY (`permission`) REFERENCES `permission` (`name`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `rolePermission_ibfk_3` FOREIGN KEY (`eventType`) REFERENCES `eventType` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "rolePermission",
    pub struct OldRolePermission {
        pub id: i64,
        pub role: i64,
        pub permission: String,
        pub eventType: Option<String>,
    }
}

// CREATE TABLE `sectionType` (
//   `id` int(1) NOT NULL,
//   `name` varchar(20) DEFAULT NULL,
//   `choir` varchar(16) DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `sectionType`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `sectionType`
//   ADD CONSTRAINT `sectionType_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON UPDATE CASCADE;
impl_load! {
    "sectionType",
    pub struct OldSectionType {
        pub id: i64,
        pub name: Option<String>,
        pub choir: Option<String>,
    }
}

// CREATE TABLE `semester` (
//   `semester` varchar(16) NOT NULL,
//   `beginning` datetime NOT NULL,
//   `end` datetime NOT NULL,
//   `gigreq` int(11) NOT NULL DEFAULT '5'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `semester`
//   ADD PRIMARY KEY (`semester`);
impl_load! {
    "semester",
    pub struct OldSemester {
        pub semester: String,
        pub beginning: NaiveDateTime,
        pub end: NaiveDateTime,
        pub gigreq: i64,
    }
}

// CREATE TABLE `song` (
//   `id` int(11) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `title` varchar(128) NOT NULL,
//   `info` text NOT NULL,
//   `current` tinyint(1) NOT NULL DEFAULT '0',
//   `key` enum('?','A♭','a♭','A','a','a♯','B♭','b♭','B','b','C♭','C','c','C♯','c♯','D♭','D','d','d♯','E♭','e♭','E','e','F','f','F♯','f♯','G♭','G','g','g♯') CHARACTER SET utf8 COLLATE utf8_bin NOT NULL DEFAULT '?',
//   `pitch` enum('?','A♭','A','A♯','B♭','B','C','C♯','D♭','D','D♯','E♭','E','F','F♯','G♭','G','G♯') CHARACTER SET utf8 COLLATE utf8_bin NOT NULL DEFAULT '?'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `song`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `song`
//   ADD CONSTRAINT `song_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);
impl_load! {
    "song",
    pub struct OldSong {
        pub id: i64,
        pub choir: String,
        pub title: String,
        pub info: String,
        pub current: bool,
        pub key: String,
        pub pitch: String,
    }
}

// CREATE TABLE `songLink` (
//   `id` int(11) NOT NULL,
//   `type` varchar(16) NOT NULL,
//   `name` varchar(128) NOT NULL,
//   `target` varchar(128) NOT NULL,
//   `song` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `songLink`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `type` (`type`),
//   ADD KEY `song` (`song`);

// ALTER TABLE `songLink`
//   ADD CONSTRAINT `songLink_ibfk_1` FOREIGN KEY (`type`) REFERENCES `mediaType` (`typeid`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `songLink_ibfk_2` FOREIGN KEY (`song`) REFERENCES `song` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "songLink",
    pub struct OldSongLink {
        pub id: i64,
        pub type_: String,
        pub name: String,
        pub target: String,
        pub song: i64,
    }
}

// CREATE TABLE `tie` (
//   `id` int(11) NOT NULL DEFAULT '0',
//   `status` varchar(20) CHARACTER SET utf8 NOT NULL DEFAULT 'Circulating',
//   `comments` varchar(200) CHARACTER SET utf8 DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=latin1;

// ALTER TABLE `tie`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `status` (`status`);

// ALTER TABLE `tie`
//   ADD CONSTRAINT `tie_ibfk_1` FOREIGN KEY (`status`) REFERENCES `tieStatus` (`name`) ON UPDATE CASCADE;
impl_load! {
    "tie",
    pub struct OldTie {
        pub id: i64,
        pub status: String,
        pub comments: Option<String>,
    }
}

// CREATE TABLE `tieBorrow` (
//   `id` int(11) NOT NULL,
//   `member` varchar(50) NOT NULL,
//   `tie` int(11) NOT NULL,
//   `dateOut` date NOT NULL,
//   `dateIn` date DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `tieBorrow`
//   ADD PRIMARY KEY (`id`),
//   ADD KEY `member` (`member`),
//   ADD KEY `tie` (`tie`);

// ALTER TABLE `tieBorrow`
//   ADD CONSTRAINT `tieBorrow_ibfk_1` FOREIGN KEY (`member`) REFERENCES `member` (`email`) ON DELETE NO ACTION ON UPDATE CASCADE,
//   ADD CONSTRAINT `tieBorrow_ibfk_2` FOREIGN KEY (`tie`) REFERENCES `tie` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "tieBorrow",
    pub struct OldTieBorrow {
        pub id: i64,
        pub member: String,
        pub tie: i64,
        pub dateOut: NaiveDate,
        pub dateIn: Option<NaiveDate>,
    }
}

// CREATE TABLE `tieStatus` (
//   `name` varchar(20) CHARACTER SET utf8 NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=latin1;

// ALTER TABLE `tieStatus`
//   ADD PRIMARY KEY (`name`);
impl_load! {
    "tieStatus",
    pub struct OldTieStatus {
        pub name: String,
    }
}

// CREATE TABLE `todo` (
//   `id` int(11) NOT NULL,
//   `text` text NOT NULL,
//   `completed` tinyint(1) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `todo`
//   ADD PRIMARY KEY (`id`);
impl_load! {
    "todo",
    pub struct OldTodo {
        pub id: i64,
        pub text: String,
        pub completed: bool,
    }
}

// CREATE TABLE `todoMembers` (
//   `memberID` varchar(50) NOT NULL,
//   `todoID` int(11) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `todoMembers`
//   ADD KEY `memberID` (`memberID`),
//   ADD KEY `todoID` (`todoID`);
impl_load! {
    "todoMembers",
    pub struct OldTodoMembers {
        pub memberID: String,
        pub todoID: i64,
    }
}

// CREATE TABLE `transaction` (
//   `memberID` varchar(50) NOT NULL,
//   `choir` varchar(16) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   `amount` int(4) NOT NULL,
//   `description` varchar(500) NOT NULL,
//   `transactionID` int(11) NOT NULL,
//   `semester` varchar(16) DEFAULT NULL,
//   `type` varchar(20) NOT NULL DEFAULT 'deposit',
//   `resolved` tinyint(1) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `transaction`
//   ADD PRIMARY KEY (`transactionID`),
//   ADD KEY `memberID` (`memberID`),
//   ADD KEY `type` (`type`),
//   ADD KEY `semester` (`semester`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `transaction`
//   ADD CONSTRAINT `transaction_ibfk_1` FOREIGN KEY (`memberID`) REFERENCES `member` (`email`) ON DELETE CASCADE ON UPDATE CASCADE,
//   ADD CONSTRAINT `transaction_ibfk_2` FOREIGN KEY (`type`) REFERENCES `transacType` (`id`),
//   ADD CONSTRAINT `transaction_ibfk_3` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE SET NULL ON UPDATE CASCADE,
//   ADD CONSTRAINT `transaction_ibfk_4` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`);
impl_load! {
    "transaction",
    pub struct OldTransaction {
        pub memberID: String,
        pub choir: String,
        pub time: NaiveDateTime,
        pub amount: i64,
        pub description: String,
        pub transactionID: i64,
        pub semester: Option<String>,
        pub type_: String,
        pub resolved: bool,
    }
}

// CREATE TABLE `transacType` (
//   `id` varchar(20) CHARACTER SET utf8 NOT NULL,
//   `name` varchar(40) CHARACTER SET utf8 NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=latin1;

// ALTER TABLE `transacType`
//   ADD PRIMARY KEY (`id`);
impl_load! {
    "transacType",
    pub struct OldTransacType {
        pub id: String,
        pub name: String,
    }
}

// CREATE TABLE `uniform` (
//   `id` varchar(20) NOT NULL,
//   `choir` varchar(16) NOT NULL DEFAULT '',
//   `name` varchar(20) NOT NULL,
//   `description` text NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `uniform`
//   ADD PRIMARY KEY (`id`,`choir`),
//   ADD KEY `choir` (`choir`);

// ALTER TABLE `uniform`
//   ADD CONSTRAINT `uniform_ibfk_1` FOREIGN KEY (`choir`) REFERENCES `choir` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
impl_load! {
    "uniform",
    pub struct OldUniform {
        pub id: String,
        pub choir: String,
        pub name: String,
        pub description: String,
    }
}

// CREATE TABLE `variables` (
//   `semester` varchar(16) NOT NULL,
//   `duesAmount` int(11) NOT NULL,
//   `tieDeposit` int(11) NOT NULL,
//   `lateFee` int(11) NOT NULL,
//   `gigCheck` tinyint(1) NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;

// ALTER TABLE `variables`
//   ADD KEY `variable_validSemester` (`semester`);

// ALTER TABLE `variables`
//   ADD CONSTRAINT `variable_validSemester` FOREIGN KEY (`semester`) REFERENCES `semester` (`semester`) ON DELETE CASCADE ON UPDATE CASCADE;
// COMMIT;
impl_load! {
    "variables",
    pub struct OldVariables {
        pub semester: String,
        pub duesAmount: i64,
        pub tieDeposit: i64,
        pub lateFee: i64,
        pub gigCheck: bool,
    }
}
