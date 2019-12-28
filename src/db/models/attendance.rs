use chrono::Local;
use db::models::member::MemberForSemester;
use db::schema::attendance::dsl::*;
use db::schema::{active_semester, event, member as member_dsl};
use db::{ActiveSemester, Attendance, AttendanceForm, Event, Member, NewAttendance};
use diesel::prelude::*;
use error::*;
use std::collections::HashMap;

impl Attendance {
    pub fn load(
        given_member: &str,
        event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<Attendance>> {
        attendance
            .filter(member.eq(given_member).and(event.eq(event_id)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    fn ensure_exists_for_member_at_event(
        given_member: &str,
        given_event: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        if Self::load(given_member, given_event, conn)?.is_some() {
            Ok(())
        } else {
            Err(GreaseError::BadRequest(format!(
                "No attendance exists for member {} at event {}. (Are they inactive?)",
                given_member, given_event
            )))
        }
    }

    pub fn load_for_event(
        event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<(Attendance, MemberForSemester)>> {
        use db::schema::member::dsl::{first_name, last_name};

        Event::load(event_id, conn)?;

        attendance
            .inner_join(member_dsl::table.inner_join(active_semester::table))
            .filter(event.eq(event_id))
            .order_by((last_name, first_name))
            .load::<(Attendance, (Member, ActiveSemester))>(conn)
            .map(|mut rows| {
                rows.dedup_by_key(|(attends, (_member, _semester))| {
                    (attends.event, attends.member.clone())
                });
                rows.into_iter()
                    .map(|(attends, (given_member, given_active_semester))| {
                        (
                            attends,
                            MemberForSemester {
                                member: given_member,
                                active_semester: Some(given_active_semester),
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_event_separate_by_section(
        given_event_id: i32,
        conn: &MysqlConnection,
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
        given_member: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<(Event, Attendance)>> {
        event::table
            .inner_join(attendance)
            .filter(
                member
                    .eq(given_member)
                    .and(event::semester.eq(given_semester)),
            )
            .order_by(event::call_time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create_for_new_member(
        given_member: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let now = Local::now().naive_local();
        let all_events_for_semester = Event::load_all_for_current_semester(conn)?;
        let new_attendances = all_events_for_semester
            .into_iter()
            .map(|event_with_gig| NewAttendance {
                event: event_with_gig.event.id,
                should_attend: if now > event_with_gig.event.call_time {
                    false
                } else {
                    event_with_gig.event.default_attend
                },
                member: given_member.to_owned(),
            })
            .collect::<Vec<NewAttendance>>();

        diesel::insert_or_ignore_into(attendance)
            .values(&new_attendances)
            .execute(conn)?;

        Ok(())
    }

    pub fn create_for_new_event(event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        let parent_event = Event::load(event_id, conn)?.event;
        let semester_members = MemberForSemester::load_all(&parent_event.semester, conn)?;

        diesel::insert_into(attendance)
            .values(
                &semester_members
                    .into_iter()
                    .map(|member_for_semester| NewAttendance {
                        event: event_id,
                        member: member_for_semester.member.email,
                        should_attend: parent_event.default_attend,
                    })
                    .collect::<Vec<NewAttendance>>(),
            )
            .execute(conn)?;

        Ok(())
    }

    pub fn excuse_unconfirmed(event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        diesel::update(attendance.filter(event.eq(event_id).and(confirmed.eq(false))))
            .set(should_attend.eq(false))
            .execute(conn)?;

        Ok(())
    }

    // TODO: don't allow updates for inactive members (NO RSVP'ing)
    pub fn update(
        event_id: i32,
        given_member: &str,
        attendance_form: &AttendanceForm,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        Self::ensure_exists_for_member_at_event(given_member, event_id, conn)?;

        diesel::update(attendance.filter(member.eq(given_member).and(event.eq(event_id))))
            .set(attendance_form)
            .execute(conn)?;

        Ok(())
    }
}
