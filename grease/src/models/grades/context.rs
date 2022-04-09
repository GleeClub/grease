use async_graphql::Result;
use time::Duration;
use std::collections::HashMap;
use crate::models::event::Event;
use crate::models::event::gig::Gig;
use crate::models::event::absence_request::AbsenceRequest;
use crate::models::event::attendance::Attendance;
use crate::models::grades::week::{EventWithAttendance, WeekOfAttendances};
use crate::db_conn::DbConn;

#[derive(Default)]
pub struct AttendanceContext {
    pub attendance: Option<Attendance>,
    pub absence_request: Option<AbsenceRequest>,
    pub rsvp_issue: Option<String>,
}

impl AttendanceContext {
    pub fn deny_credit(&self) -> bool {
        if let Some(attendance) = self.attendance {
            let approved_absence = self.absence_request.map(|request| request.state == AbsenceRequestState::Approved).unwrap_or(false);
            attendance.should_attend && !self.did_attend && !approved_absence
        } else {
            false
        }
    }
}

pub struct GradesContext {
    pub semester: Semester,
    pub events: Vec<(Event, Option<Gig>)>,
    pub attendance: HashMap<isize, HashMap<String, AttendanceContext>>,
}

impl GradesContext {
    pub async fn for_member_during_semester(email: &str, semester: &str, conn: &DbConn) -> Result<Self> {
        let semester = Semester::with_name(semester, conn).await?;
        let events = Event::for_semester(semester, conn).await?;
        let mut gigs = Gig::for_semester(semester, conn).await?;
        let event_types: HashMap<isize, &str> = events.iter().map(|event| (event.id, &event.r#type)).collect();
        let attendance = AttendanceContext::for_member_during_semester(email, semester, &event_types, conn).await?;

        Ok(Self {
            semester,
            events: events.into_iter().map(|event| (event, gigs.drain_filter(|gig| gig.event == event.id).next())).collect(),
            attendance,
        })
    }

    pub fn weeks_of_attendance(&self, email: &str) -> impl Iterator<Item = WeekOfAttendances> {
        let start_of_semester = self.events.first().and_then(|event| event.call_time).unwrap_or(semester.start_date);
        let end_of_semester = self.events.last().and_then(|event| event.call_time).unwrap_or(semester.end_date);
        let first_sunday = start_of_semester - Duration::days(start_of_semester.weekday().number_from_sunday() - 1);

        std::iter::successors::successors(
            Some(first_sunday),
            |sunday| Some(sunday + Duration::weeks(1)).filter(|s| s <= end_of_semester)
        ).map(|sunday| {
            let next_sunday = sunday + Duration::weeks(1);

            WeekOfAttendances {
                events: self.events.iter()
                    .skip_while(|(event, _gig)| event.call_time < sunday)
                    .take_while(|(event, _gig)| event.call_time < next_sunday)
                    .map(|(event, gig)| EventWithAttendance {
                        event,
                        gig,
                        attendance: self.attendance.get(&event.id).and_then(|members| members.get(email)),
                    })
                    .collect(),
            }
        })
    }
}

impl AttendanceContext {
    async fn for_member_during_semester(email: &str, semester: &str, event_types: &HashMap<isize, &str>, conn: &DbConn) -> Result<HashMap<isize, HashMap<String, Self>>> {
        let is_active = sqlx::query!(
            "SELECT member FROM active_semester WHERE member = ? AND semester = ?",
                email, semester).fetch_optional(conn).await?.is_some();
        let attendances: Vec<Attendance> = sqlx::query!(
            "SELECT * FROM attendance WHERE member = ? AND event IN
             (SELECT id FROM event WHERE semester = ?)",
            email, semester
        ).fetch_all(conn).await?;
        let absence_requests: Vec<AbsenceRequest> = sqlx::query!(
            "SELECT * FROM absence_request WHERE member = ? AND event IN
             (SELECT id FROM event WHERE semester = ?)",
            email, semester
        ).fetch_all(conn).await?;

        let mut all_context: HashMap<isize, HashMap<String, Self>> = HashMap::new();

        for attendance in attendances {
            let context = context.entry(attendance.event)
                .or_default()
                .entry(attendance.member.clone())
                .or_default();
            let rsvp_issue = event_types.get(&attendance.event).and_then(|event_type| Event::rsvp_issue_for(event_type, attendance, is_active));

            context.attendance = Some(attendance);
            context.rsvp_issue = rsvp_issue;
        }

        for absence_request in absence_requests {
            all_context.entry(absence_request.event)
                .or_default()
                .entry(absence_request.member.clone())
                .or_default()
                .absence_request = Some(absence_request);
        }

        Ok(all_context)
    }
}
