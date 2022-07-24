use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_graphql::Result;
use sqlx::PgPool;
use time::Duration;

use crate::models::event::absence_request::{AbsenceRequest, AbsenceRequestStatus};
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
                .map(|request| request.state == AbsenceRequestStatus::Approved)
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
    pub attendance: HashMap<i64, HashMap<String, AttendanceContext>>,
}

impl GradesContext {
    pub async fn for_members_during_semester(
        emails: &Vec<String>,
        semester: &str,
        pool: &PgPool,
    ) -> Result<Self> {
        let semester = Semester::with_name(semester, pool).await?;
        let events = Event::for_semester(&semester.name, pool)
            .await?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let mut gigs = Gig::for_semester(&semester.name, pool).await?;
        let event_map: HashMap<i64, Arc<Event>> = events
            .iter()
            .map(|event| (event.id, Arc::clone(event)))
            .collect();
        let attendance = AttendanceContext::for_members_during_semester(
            emails,
            &semester.name,
            &event_map,
            pool,
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
            .map(|(event, _gig)| event.call_time.0.date())
            .unwrap_or(self.semester.start_date.0);
        let end_of_semester = self
            .events
            .last()
            .map(|(event, _gig)| event.call_time.0.date())
            .unwrap_or(self.semester.end_date.0);
        let days_after_sunday = start_of_semester.weekday().number_from_sunday() - 1;
        let first_sunday = start_of_semester - Duration::days(days_after_sunday as i64);

        std::iter::successors(Some(first_sunday), move |sunday| {
            Some(*sunday + Duration::weeks(1)).filter(|s| s <= &end_of_semester)
        })
        .map(|sunday| {
            let next_sunday = sunday + Duration::weeks(1);

            WeekOfAttendances {
                events: self
                    .events
                    .iter()
                    .skip_while(|(event, _gig)| event.call_time.0.date() < sunday)
                    .take_while(|(event, _gig)| event.call_time.0.date() < next_sunday)
                    .map(|(event, gig)| EventWithAttendance {
                        event: Arc::clone(event),
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
        emails: &Vec<String>,
        semester: &str,
        events: &HashMap<i64, Arc<Event>>,
        pool: &PgPool,
    ) -> Result<HashMap<i64, HashMap<String, Self>>> {
        let active_members: HashSet<String> = sqlx::query_scalar!(
            "SELECT member FROM active_semesters
             WHERE member = ANY($1) AND semester = $2",
            &emails[..],
            semester
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect();

        let attendances: Vec<Attendance> = sqlx::query_as!(
            Attendance,
            "SELECT should_attend, did_attend, confirmed, minutes_late, member, event
             FROM attendance
             WHERE member = Any($1) AND event IN
             (SELECT id FROM events WHERE semester = $2)",
            &emails[..],
            semester
        )
        .fetch_all(pool)
        .await?;

        let absence_requests: Vec<AbsenceRequest> = sqlx::query_as!(
            AbsenceRequest,
            "SELECT member, event, \"time\" as \"time: _\", reason, state as \"state: _\"
             FROM absence_requests
             WHERE member = ANY($1) AND event IN
             (SELECT id FROM events WHERE semester = $2)",
            emails,
            semester
        )
        .fetch_all(pool)
        .await?;

        let mut all_context: HashMap<i64, HashMap<String, Self>> = HashMap::new();

        for attendance in attendances {
            let context = all_context
                .entry(attendance.event)
                .or_default()
                .entry(attendance.member.clone())
                .or_default();
            let is_active = active_members.contains(&attendance.member);
            let rsvp_issue = if let Some(event) = events.get(&attendance.event) {
                event.rsvp_issue_for(Some(&attendance), is_active)
            } else {
                None
            };

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
