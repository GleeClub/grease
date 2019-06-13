use db::models::event::EventWithGig;
use db::models::member::MemberForSemester;
use db::models::*;
use db::schema::attendance::dsl::*;
use db::schema::member::dsl::{first_name, last_name};
use db::schema::AbsenceRequestState;
use db::schema::{active_semester, attendance, event, member};
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::*;
use std::collections::HashMap;

impl Attendance {
    pub fn load(
        given_member_email: &str,
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Attendance> {
        attendance
            .filter(member.eq(given_member_email).and(event.eq(given_event_id)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "attendance for member {} for event {} not found",
                given_member_email, given_event_id
            )))
    }

    pub fn load_for_event(
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<(EventWithGig, Vec<(Attendance, MemberForSemester)>)> {
        let found_event = Event::load(given_event_id, conn)?;
        let attendance_data = attendance::table
            .inner_join(member::table.inner_join(active_semester::table))
            .filter(attendance::dsl::event.eq(&given_event_id))
            .order((first_name, last_name)) // TODO: Which way is this supposed to go (first or last first)?
            .load::<(Attendance, (Member, ActiveSemester))>(conn)
            .map_err(GreaseError::DbError)?;

        Ok((
            found_event,
            attendance_data
                .into_iter()
                .map(
                    |(found_attendance, (found_member, found_active_semester))| {
                        (
                            found_attendance,
                            MemberForSemester {
                                member: found_member,
                                active_semester: found_active_semester,
                            },
                        )
                    },
                )
                .collect(),
        ))
    }

    pub fn load_for_event_separate_by_section(
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<(
        EventWithGig,
        HashMap<Option<String>, (Attendance, MemberForSemester)>,
    )> {
        let (found_event, pairs) = Attendance::load_for_event(given_event_id, conn)?;
        let sorted_attendance = pairs
            .into_iter()
            .map(|(member_attendance, member_for_semester)| {
                (
                    member_for_semester.active_semester.section.clone(),
                    (member_attendance, member_for_semester),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok((found_event, sorted_attendance))
    }

    pub fn load_for_member_at_event(
        given_member_email: &str,
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Attendance> {
        attendance
            .filter(event.eq(given_event_id).and(member.eq(given_member_email)))
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_member_at_all_events(
        given_member_email: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<(Event, Attendance)>> {
        let current_semester = Semester::load_current(conn)?;
        event::table
            .inner_join(attendance::table)
            .filter(
                attendance::dsl::member
                    .eq(given_member_email)
                    .and(event::dsl::semester.eq(&current_semester.name)),
            )
            .load::<(Event, Attendance)>(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_member_at_all_events_of_type(
        given_member_email: &str,
        event_type: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<(Attendance, Event)>> {
        let current_semester = Semester::load_current(conn)?;
        attendance::table
            .inner_join(event::table)
            .filter(
                member
                    .eq(&given_member_email)
                    .and(event::dsl::type_.eq(event_type))
                    .and(event::dsl::semester.eq(&current_semester.name)),
            )
            .load::<(Attendance, Event)>(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create_for_new_member(
        given_member_email: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let new_attendances = Event::load_all_for_current_semester(conn)?
            .into_iter()
            .map(|event_with_gig| NewAttendance {
                event: event_with_gig.event.id,
                member: given_member_email.to_owned(),
            })
            .collect::<Vec<NewAttendance>>();
        diesel::insert_into(attendance)
            .values(&new_attendances)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn create_for_new_event(given_event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        let event_semester = Event::load(given_event_id, conn)?.event.semester;
        let semester_members = MemberForSemester::load_all(&event_semester, conn)?;

        let new_attendances = semester_members
            .into_iter()
            .map(|member_for_semester| NewAttendance {
                event: given_event_id,
                member: member_for_semester.member.email,
            })
            .collect::<Vec<NewAttendance>>();
        diesel::insert_into(attendance)
            .values(&new_attendances)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn update(
        given_event_id: i32,
        given_member_email: &str,
        attendance_form: &AttendanceForm,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        diesel::update(
            attendance.filter(member.eq(given_member_email).and(event.eq(given_event_id))),
        )
        .set(attendance_form)
        .execute(conn)
        .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn is_excused(&self, conn: &MysqlConnection) -> GreaseResult<bool> {
        AbsenceRequest::load(&self.member, self.event, conn).map(|absence_request| {
            absence_request
                .map(|ar| ar.state == AbsenceRequestState::Approved)
                .unwrap_or(false)
        })
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "attendance"]
pub struct NewAttendance {
    pub event: i32,
    pub member: String,
}

#[derive(AsChangeset, Debug, Serialize)]
#[table_name = "attendance"]
pub struct AttendanceForm {
    pub should_attend: bool,
    pub did_attend: Option<bool>,
    pub minutes_late: i32,
    pub confirmed: bool,
}
