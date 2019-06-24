use chrono::{Local, NaiveDateTime};
use db::*;
use error::*;
use db::models::member::MemberForSemester;
use pinto::query_builder::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

impl Attendance {
    pub fn load<C: Connection>(
        member: &str,
        event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<Attendance> {
        conn.first(
            &Self::filter(&format!("member = '{}' AND event = {}", member, event_id)),
            format!("Attendance for member {} for event {} not found.", member, event_id)
        )
    }

    pub fn load_for_event<C: Connection>(
        event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<Vec<(Attendance, MemberForSemester)>> {
        // to ensure that the event exists
        let _found_event = Event::load(event_id, conn)?;

        conn.load_as::<AttendanceMemberRow, (Attendance, MemberForSemester)>(
            Select::new(Self::table_name())
                .join(
                    Member::table_name(),
                    &format!("{}.member", Self::table_name()),
                    "email",
                    Join::Inner,
                )
                .join(
                    ActiveSemester::table_name(),
                    &format!("{}.member", Self::table_name()),
                    &format!("{}.member", ActiveSemester::table_name()),
                    Join::Inner,
                )
                .fields(AttendanceMemberRow::field_names())
                .filter(&format!("event = {}", event_id))
                .order_by("last_name, first_name", Order::Asc)
        )
    }

    pub fn load_for_event_for_section<C: Connection>(
        event_id: i32,
        section: Option<&str>,
        conn: &mut C,
    ) -> GreaseResult<Vec<(Attendance, MemberForSemester)>> {
        let _found_event = Event::load(event_id, conn)?;

        conn.load_as::<AttendanceMemberRow, (Attendance, MemberForSemester)>(
            Select::new(Self::table_name())
                .join(
                    Member::table_name(),
                    &format!("{}.member", Self::table_name()),
                    "email",
                    Join::Inner,
                )
                .join(
                    ActiveSemester::table_name(),
                    &format!("{}.member", Self::table_name()),
                    &format!("{}.member", ActiveSemester::table_name()),
                    Join::Inner,
                )
                .fields(AttendanceMemberRow::field_names())
                .filter(&format!("event = {}", event_id))
                .filter(&format!("section = {}", to_value(section)))
                .order_by("last_name, first_name", Order::Asc)
        )
    }

    pub fn load_for_event_separate_by_section<C: Connection>(
        given_event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<HashMap<Option<String>, Vec<(Attendance, MemberForSemester)>>> {
        Attendance::load_for_event(given_event_id, conn).map(|pairs| {
            let mut section_attendance: HashMap<Option<String>, Vec<(_, _)>> = HashMap::new();
            for (member_attendance, member_for_semester) in pairs {
                section_attendance
                    .entry(member_for_semester.active_semester.section.clone())
                    .or_default()
                    .push((member_attendance, member_for_semester));
            }

            section_attendance
        })
    }

    pub fn load_for_member_at_all_events<C: Connection>(
        member: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<(Event, Attendance)>> {
        conn.load_as::<EventAttendanceRow, (Event, Attendance)>(
            Select::new(Event::table_name())
                .join(Attendance::table_name(), "id", "event", Join::Inner)
                .fields(EventAttendanceRow::field_names())
                .filter(&format!("member = '{}'", member))
                .filter(&format!("semester = '{}'", semester))
        )
    }

    pub fn load_for_member_at_all_events_of_type<C: Connection>(
        member: &str,
        event_type: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<(Event, Attendance)>> {
        let current_semester = Semester::load_current(conn)?;
        conn.load_as::<EventAttendanceRow, (Event, Attendance)>(
            Select::new(Event::table_name())
                .join(Attendance::table_name(), "id", "event", Join::Inner)
                .fields(EventAttendanceRow::field_names())
                .filter(&format!("member = '{}'", member))
                .filter(&format!("semester = '{}'", &current_semester.name))
                .filter(&format!("type = '{}'", event_type))
        )
    }

    pub fn create_for_new_member(
        given_member_email: &str,
        conn: &mut DbTransaction,
    ) -> GreaseResult<()> {
        let now = Local::now().naive_local();
        Event::load_all_for_current_semester(conn)?
            .into_iter()
            .map(|event_with_gig| NewAttendance {
                event: event_with_gig.event.id,
                should_attend: if now > event_with_gig.event.call_time {
                    false
                } else {
                    event_with_gig.event.default_attend
                },
                member: given_member_email.to_owned(),
            })
            .map(|new_attendance| new_attendance.insert(conn))
            .collect::<GreaseResult<()>>()
    }

    pub fn create_for_new_event<C: Connection>(
        event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<()> {
        let event = Event::load(event_id, conn)?.event;
        let semester_members = MemberForSemester::load_all(&event.semester, conn)?;

        semester_members
            .into_iter()
            .map(|member_for_semester| NewAttendance {
                event: event_id,
                member: member_for_semester.member.email,
                should_attend: event.default_attend,
            })
            .map(|new_attendance| new_attendance.insert(conn))
            .collect::<GreaseResult<()>>()
    }

    pub fn excuse_unconfirmed<C: Connection>(event_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.update_opt(
            Update::new(Self::table_name())
                .filter(&format!("event = {} AND confirmed = false", event_id))
                .set("should_attend", "false")
        )
    }

    pub fn update<C: Connection>(
        event_id: i32,
        member: &str,
        attendance_form: &AttendanceForm,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("member = '{}'", member))
                .filter(&format!("event = {}", event_id))
                .set("should_attend", &to_value(&attendance_form.should_attend))
                .set("did_attend", &to_value(&attendance_form.did_attend))
                .set("minutes_late", &to_value(&attendance_form.minutes_late))
                .set("confirmed", &to_value(&attendance_form.confirmed)),
            format!("No attendance exists for member {} at event {}. (Are they inactive?)", member, event_id)
        )
    }

    pub fn is_excused<C: Connection>(&self, conn: &mut C) -> GreaseResult<bool> {
        AbsenceRequest::load(&self.member, self.event, conn).map(|absence_request| {
            absence_request
                .map(|request| request.state == AbsenceRequestState::Approved)
                .unwrap_or(false)
        })
    }
}

#[derive(grease_derive::TableName, grease_derive::Insertable, Serialize, Deserialize)]
#[table_name = "attendance"]
pub struct NewAttendance {
    pub event: i32,
    pub should_attend: bool,
    pub member: String,
}

#[derive(Debug, Serialize, Deserialize, grease_derive::Extract)]
pub struct AttendanceForm {
    pub should_attend: bool,
    pub did_attend: bool,
    pub minutes_late: i32,
    pub confirmed: bool,
}

#[derive(grease_derive::FieldNames, grease_derive::FromRow)]
pub struct EventAttendanceRow {
    pub id: i32,
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<String>,
    pub member: String,
    pub event: i32,
    pub should_attend: bool,
    pub did_attend: bool,
    pub confirmed: bool,
    pub minutes_late: i32,
}

impl Into<(Event, Attendance)> for EventAttendanceRow {
    fn into(self) -> (Event, Attendance) {
        (
            Event {
                id: self.id,
                name: self.name,
                semester: self.semester,
                type_: self.type_,
                call_time: self.call_time,
                release_time: self.release_time,
                points: self.points,
                comments: self.comments,
                location: self.location,
                gig_count: self.gig_count,
                default_attend: self.default_attend,
                section: self.section,
            },
            Attendance {
                member: self.member,
                event: self.event,
                should_attend: self.should_attend,
                did_attend: self.did_attend,
                confirmed: self.confirmed,
                minutes_late: self.minutes_late,
            },
        )
    }
}

#[derive(grease_derive::FieldNames, grease_derive::FromRow)]
pub struct AttendanceMemberRow {
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
    #[rename = "`active_semester`.`member`"]
    pub semester_member: String,
    pub semester: String,
    pub enrollment: Enrollment,
    pub section: Option<String>,
    #[rename = "`attendance`.`member`"]
    pub attendance_member: String,
    pub event: i32,
    pub should_attend: bool,
    pub did_attend: bool,
    pub confirmed: bool,
    pub minutes_late: i32,
}

impl Into<(Attendance, MemberForSemester)> for AttendanceMemberRow {
    fn into(self) -> (Attendance, MemberForSemester) {
        (
            Attendance {
                member: self.attendance_member,
                event: self.event,
                should_attend: self.should_attend,
                did_attend: self.did_attend,
                confirmed: self.confirmed,
                minutes_late: self.minutes_late,
            },
            MemberForSemester {
                member: Member {
                    email: self.email,
                    first_name: self.first_name,
                    preferred_name: self.preferred_name,
                    last_name: self.last_name,
                    pass_hash: self.pass_hash,
                    phone_number: self.phone_number,
                    picture: self.picture,
                    passengers: self.passengers,
                    location: self.location,
                    about: self.about,
                    major: self.major,
                    minor: self.minor,
                    hometown: self.hometown,
                    arrived_at_tech: self.arrived_at_tech,
                    gateway_drug: self.gateway_drug,
                    conflicts: self.conflicts,
                    dietary_restrictions: self.dietary_restrictions,
                },
                active_semester: ActiveSemester {
                    member: self.semester_member,
                    semester: self.semester,
                    enrollment: self.enrollment,
                    section: self.section,
                },
            },
        )
    }
}
