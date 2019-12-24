use chrono::{Local, NaiveDateTime};
use db::models::member::MemberForSemester;
use db::{ActiveSemester, Event, Attendance, NewAttendance};
use diesel::prelude::*;
use error::*;
use std::collections::HashMap;
use db::schema::attendance::dsl::*;
use db::schema::{member as member_dsl, active_semester};

impl Attendance {
    pub fn load(
        given_member: &str,
        event_id: i32,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<Option<Attendance>> {
        attendance.filter(member.eq(given_member).and(event.eq(event_id)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_event(
        event_id: i32,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<Vec<(Attendance, MemberForSemester)>> {
        use db::schema::member::dsl::{first_name, last_name};

        Event::load(event_id, conn)?
            .ok_or(GreaseError::BadRequest(format!("No event exists with id {}.", event_id)))?;

        attendance.inner_join(member_dsl::table.inner_join(active_semester::table))
            .filter(event.eq(event_id))
            .order_by((last_name, first_name))
            .load::<(Attendance, MemberForSemester)>(conn)
            .map(|mut rows| {
                rows.dedup_by_key(|(attends, _member)| (attends.event, attends.member.clone()));
                rows
            })
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_event_for_section(
        event_id: i32,
        section: Option<&str>,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<Vec<(Attendance, MemberForSemester)>> {
        let _found_event = Event::load(event_id, conn)?;

        conn.load_as::<AttendanceMemberRow, (Attendance, MemberForSemester)>(
            Select::new(Attendance::table_name())
                .join(
                    Member::table_name(),
                    &format!("{}.member", Attendance::table_name()),
                    "email",
                    Join::Inner,
                )
                .join(
                    ActiveSemester::table_name(),
                    &format!("{}.member", Attendance::table_name()),
                    &format!("{}.member", ActiveSemester::table_name()),
                    Join::Inner,
                )
                .fields(AttendanceMemberRow::field_names())
                .filter(&format!("event = {}", event_id))
                .filter(&format!("section = {}", to_value(section)))
                .order_by("last_name, first_name", Order::Asc),
        )
    }

    pub fn load_for_event_separate_by_section(
        given_event_id: i32,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<HashMap<String, Vec<(Attendance, MemberForSemester)>>> {
        let attendance_pairs = Attendance::load_for_event(given_event_id, conn)?;
        let mut section_attendance: HashMap<String, Vec<(_, _)>> = HashMap::new();

        for (member_attendance, member_for_semester) in attendance_pairs {
            let member_section = member_for_semester
                .active_semester
                .as_ref()
                .and_then(|active_semester| active_semester.section.clone())
                .unwrap_or("Unsorted".to_owned());
            section_attendance
                .entry(member_section)
                .or_default()
                .push((member_attendance, member_for_semester));
        }

        Ok(section_attendance)
    }

    pub fn load_for_member_at_all_events(
        member: &str,
        semester: &str,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<Vec<(Event, Attendance)>> {
        conn.load_as::<EventAttendanceRow, (Event, Attendance)>(
            Select::new(Event::table_name())
                .join(Attendance::table_name(), "id", "event", Join::Inner)
                .fields(EventAttendanceRow::field_names())
                .filter(&format!("member = '{}'", member))
                .filter(&format!("semester = '{}'", semester))
                .order_by("call_time", Order::Asc),
        )
    }

    pub fn create_for_new_member(member: &str, conn: &mut DbTransaction) -> GreaseResult<()> {
        let now = Local::now().naive_local();
        for event_with_gig in Event::load_all_for_current_semester(conn)? {
            let new_attendance = NewAttendance {
                event: event_with_gig.event.id,
                should_attend: if now > event_with_gig.event.call_time {
                    false
                } else {
                    event_with_gig.event.default_attend
                },
                member: member.to_owned(),
            };

            if conn
                .first_opt::<Attendance>(&Attendance::filter(&format!(
                    "member = '{}' AND event = {}",
                    member, new_attendance.event
                )))?
                .is_none()
            {
                new_attendance.insert(conn)?;
            }
        }

        Ok(())
    }

    pub fn create_for_new_event(event_id: i32, conn: &mut MysqlConnection) -> GreaseResult<()> {
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

    pub fn excuse_unconfirmed(event_id: i32, conn: &mut MysqlConnection) -> GreaseResult<()> {
        conn.update_opt(
            Update::new(Attendance::table_name())
                .filter(&format!("event = {} AND confirmed = false", event_id))
                .set("should_attend", "false"),
        )
    }

    // TODO: don't allow updates for inactive members (NO RSVP'ing)
    pub fn update(
        event_id: i32,
        member: &str,
        attendance_form: &AttendanceForm,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<()> {
        conn.update(
            Update::new(Attendance::table_name())
                .filter(&format!("member = '{}'", member))
                .filter(&format!("event = {}", event_id))
                .set("should_attend", &to_value(&attendance_form.should_attend))
                .set("did_attend", &to_value(&attendance_form.did_attend))
                .set("minutes_late", &to_value(&attendance_form.minutes_late))
                .set("confirmed", &to_value(&attendance_form.confirmed)),
            format!(
                "No attendance exists for member {} at event {}. (Are they inactive?)",
                member, event_id
            ),
        )
    }
}
