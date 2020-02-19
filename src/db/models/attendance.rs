use chrono::Local;
use db::models::event::EventWithGig;
use db::models::member::MemberForSemester;
use db::schema::attendance::dsl::*;
use db::schema::{
    absence_request, active_semester, event, gig, member as member_dsl, AbsenceRequestState,
};
use db::{
    AbsenceRequest, ActiveSemester, Attendance, AttendanceForm, Event, Gig, Member, NewAttendance,
};
use diesel::prelude::*;
use error::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct MemberAttendance {
    #[serde(flatten)]
    pub event: EventWithGig,
    pub attendance: Option<Attendance>,
    #[serde(rename = "absenceRequest")]
    pub absence_request: Option<AbsenceRequest>,
    #[serde(rename = "rsvpIssue")]
    pub rsvp_issue: Option<String>,
}

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
        let mut rows = attendance
            .inner_join(member_dsl::table.inner_join(active_semester::table))
            .filter(event.eq(event_id))
            .order_by((last_name, first_name))
            .load::<(Attendance, (Member, ActiveSemester))>(conn)?;
        rows.dedup_by_key(|(attends, _)| (attends.event, attends.member.clone()));

        Ok(rows
            .into_iter()
            .map(|(attends, (given_member, given_active_semester))| {
                (
                    attends,
                    MemberForSemester {
                        member: given_member,
                        active_semester: Some(given_active_semester),
                    },
                )
            })
            .collect::<Vec<_>>())
    }

    pub fn load_for_member_at_event(
        given_member: &Member,
        is_active: bool,
        event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberAttendance> {
        let (e, g, a, r) = event::table
            .left_outer_join(gig::table)
            .left_outer_join(attendance)
            .left_outer_join(
                absence_request::table.on(absence_request::event
                    .eq(event::id)
                    .and(absence_request::member.eq(&given_member.email))),
            )
            .filter(event::id.eq(event_id).and(member.eq(&given_member.email)))
            .first::<(
                Event,
                Option<Gig>,
                Option<Attendance>,
                Option<AbsenceRequest>,
            )>(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "No event exists with id {}.",
                event_id
            )))?;

        let rsvp_issue = e.rsvp_issue(a.as_ref(), is_active);
        Ok(MemberAttendance {
            event: EventWithGig { event: e, gig: g },
            attendance: a,
            absence_request: r,
            rsvp_issue,
        })
    }
    pub fn load_for_member_at_all_events(
        given_member: &Member,
        is_active: bool,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<MemberAttendance>> {
        let rows = event::table
            .left_outer_join(gig::table)
            .left_outer_join(attendance)
            .left_outer_join(
                absence_request::table.on(absence_request::event
                    .eq(event::id)
                    .and(absence_request::member.eq(&given_member.email))),
            )
            .filter(
                member
                    .eq(&given_member.email)
                    .and(event::semester.eq(given_semester)),
            )
            .order_by(event::call_time.asc())
            .load::<(
                Event,
                Option<Gig>,
                Option<Attendance>,
                Option<AbsenceRequest>,
            )>(conn)?;

        Ok(rows
            .into_iter()
            .map(|(e, g, a, r)| {
                let rsvp_issue = e.rsvp_issue(a.as_ref(), is_active);
                MemberAttendance {
                    event: EventWithGig { event: e, gig: g },
                    attendance: a,
                    absence_request: r,
                    rsvp_issue,
                }
            })
            .collect())
    }

    pub fn create_for_new_member(given_member: &str, conn: &MysqlConnection) -> GreaseResult<()> {
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

impl MemberAttendance {
    pub fn deny_credit(&self) -> bool {
        self.should_attend() && !self.did_attend() && !self.approved_absence()
    }

    pub fn approved_absence(&self) -> bool {
        self.absence_request
            .as_ref()
            .map(|request| request.state == AbsenceRequestState::Approved)
            .unwrap_or(false)
    }

    pub fn should_attend(&self) -> bool {
        self.attendance
            .as_ref()
            .map(|a| a.should_attend)
            .unwrap_or(false)
    }

    pub fn did_attend(&self) -> bool {
        self.attendance
            .as_ref()
            .map(|a| a.did_attend)
            .unwrap_or(false)
    }

    pub fn confirmed(&self) -> bool {
        self.attendance
            .as_ref()
            .map(|a| a.confirmed)
            .unwrap_or(false)
    }

    pub fn minutes_late(&self) -> i32 {
        self.attendance
            .as_ref()
            .map(|a| a.minutes_late)
            .unwrap_or(0)
    }
}
