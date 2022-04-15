use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_graphql::Result;
use time::Duration;

use crate::db::DbConn;
use crate::models::event::absence_request::{AbsenceRequest, AbsenceRequestState};
use crate::models::event::attendance::Attendance;
use crate::models::event::gig::Gig;
use crate::models::event::Event;
use crate::models::grades::week::{EventWithAttendance, WeekOfAttendances};
use crate::models::semester::Semester;

#[derive(Default)]
pub struct AttendanceContext {
    pub attendance: Option<Attendance>,
    pub absence_request: Option<AbsenceRequest>,
    pub rsvp_issue: Option<String>,
}

impl AttendanceContext {
    pub fn deny_credit(&self) -> bool {
        if let Some(attendance) = &self.attendance {
            let approved_absence = self
                .absence_request
                .as_ref()
                .map(|request| request.state == AbsenceRequestState::Approved)
                .unwrap_or(false);
            attendance.should_attend && !attendance.did_attend && !approved_absence
        } else {
            false
        }
    }
}

pub struct GradesContext {
    pub semester: Semester,
    pub events: Vec<(Arc<Event>, Option<Gig>)>,
    pub attendance: HashMap<i32, HashMap<String, AttendanceContext>>,
}

impl GradesContext {
    pub async fn for_members_during_semester(
        emails: &Vec<&str>,
        semester: &str,
        conn: &DbConn,
    ) -> Result<Self> {
        let semester = Semester::with_name(semester, conn).await?;
        let events = Event::for_semester(&semester.name, conn)
            .await?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let mut gigs = Gig::for_semester(&semester.name, conn).await?;
        let event_map: HashMap<i32, Arc<Event>> = events
            .iter()
            .map(|event| (event.id, event.clone()))
            .collect();
        let attendance = AttendanceContext::for_members_during_semester(
            emails,
            &semester.name,
            &event_map,
            conn,
        )
        .await?;

        Ok(Self {
            semester,
            events: events
                .into_iter()
                .map(|event| {
                    let gig = gigs.drain_filter(|gig| gig.event == event.id).next();
                    (event, gig)
                })
                .collect(),
            attendance,
        })
    }

    pub fn weeks_of_attendance<'e, 'a: 'e>(
        &'a self,
        email: &'e str,
    ) -> impl Iterator<Item = WeekOfAttendances<'a>> + 'e {
        let start_of_semester = self
            .events
            .first()
            .map(|(event, _gig)| &event.call_time)
            .unwrap_or(&self.semester.start_date);
        let end_of_semester = self
            .events
            .last()
            .map(|(event, _gig)| &event.call_time)
            .unwrap_or(&self.semester.end_date);
        let days_after_sunday = start_of_semester.0.weekday().number_from_sunday() - 1;
        let first_sunday = start_of_semester.0 - Duration::days(days_after_sunday as i64);

        std::iter::successors(Some(first_sunday), move |sunday| {
            Some(*sunday + Duration::weeks(1)).filter(|s| s <= &end_of_semester.0)
        })
        .map(|sunday| {
            let next_sunday = sunday + Duration::weeks(1);

            WeekOfAttendances {
                events: self
                    .events
                    .iter()
                    .skip_while(|(event, _gig)| event.call_time.0 < sunday)
                    .take_while(|(event, _gig)| event.call_time.0 < next_sunday)
                    .map(|(event, gig)| EventWithAttendance {
                        event: event.clone(),
                        gig: gig.as_ref(),
                        attendance: self
                            .attendance
                            .get(&event.id)
                            .and_then(|members_attendance| members_attendance.get(email)),
                    })
                    .collect(),
            }
        })
    }
}

impl AttendanceContext {
    async fn for_members_during_semester(
        emails: &Vec<&str>,
        semester: &str,
        events: &HashMap<i32, Arc<Event>>,
        conn: &DbConn,
    ) -> Result<HashMap<i32, HashMap<String, Self>>> {
        let active_members: HashSet<String> = sqlx::query_scalar!(
            "SELECT member FROM active_semester
             WHERE find_in_set(member, ?) AND semester = ?",
            emails.join(","),
            semester
        )
        .fetch_all(conn)
        .await?
        .into_iter()
        .collect();

        let attendances: Vec<Attendance> = sqlx::query_as!(
            Attendance,
            "SELECT should_attend as \"should_attend: bool\", did_attend as \"did_attend: bool\",
                 confirmed as \"confirmed: bool\", minutes_late, member, event
             FROM attendance
             WHERE find_in_set(member, ?) AND event IN
             (SELECT id FROM event WHERE semester = ?)",
            emails.join(","),
            semester
        )
        .fetch_all(conn)
        .await?;

        let absence_requests: Vec<AbsenceRequest> = sqlx::query_as!(
            AbsenceRequest,
            "SELECT member, event, `time` as \"time: _\", reason, state as \"state: _\"
             FROM absence_request
             WHERE find_in_set(member, ?) AND event IN
             (SELECT id FROM event WHERE semester = ?)",
            emails.join(","),
            semester
        )
        .fetch_all(conn)
        .await?;

        let mut all_context: HashMap<i32, HashMap<String, Self>> = HashMap::new();

        for attendance in attendances {
            let context = all_context
                .entry(attendance.event)
                .or_default()
                .entry(attendance.member.clone())
                .or_default();
            let is_active = active_members.contains(&attendance.member);
            let rsvp_issue = events
                .get(&attendance.event)
                .and_then(|event| event.rsvp_issue_for(Some(&attendance), is_active));

            context.attendance = Some(attendance);
            context.rsvp_issue = rsvp_issue;
        }

        for absence_request in absence_requests {
            let context = all_context
                .entry(absence_request.event)
                .or_default()
                .entry(absence_request.member.clone())
                .or_default();
            context.absence_request = Some(absence_request);
        }

        Ok(all_context)
    }
}
