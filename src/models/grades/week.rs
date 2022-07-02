use std::sync::Arc;

use crate::models::event::absence_request::AbsenceRequestStatus;
use crate::models::event::gig::Gig;
use crate::models::event::{Event, EventType};
use crate::models::grades::context::AttendanceContext;

pub struct EventWithAttendance<'a> {
    pub event: Arc<Event>,
    pub gig: Option<&'a Gig>,
    pub attendance: Option<&'a AttendanceContext>,
}

impl<'a> EventWithAttendance<'a> {
    pub fn approved_absence(&self) -> bool {
        self.attendance
            .as_ref()
            .and_then(|a| a.absence_request.as_ref())
            .map(|ar| ar.state == AbsenceRequestStatus::Approved)
            .unwrap_or(false)
    }

    pub fn should_attend(&self) -> bool {
        self.attendance
            .as_ref()
            .and_then(|a| a.attendance.as_ref())
            .map(|a| a.should_attend)
            .unwrap_or(false)
    }

    pub fn did_attend(&self) -> bool {
        self.attendance
            .as_ref()
            .and_then(|a| a.attendance.as_ref())
            .map(|a| a.did_attend)
            .unwrap_or(false)
    }

    pub fn confirmed(&self) -> bool {
        self.attendance
            .as_ref()
            .and_then(|a| a.attendance.as_ref())
            .map(|a| a.confirmed)
            .unwrap_or(false)
    }

    pub fn minutes_late(&self) -> i64 {
        self.attendance
            .as_ref()
            .and_then(|a| a.attendance.as_ref())
            .map(|a| a.minutes_late)
            .unwrap_or(0)
    }
}

pub struct WeekOfAttendances<'a> {
    pub events: Vec<EventWithAttendance<'a>>,
}

impl<'a> WeekOfAttendances<'a> {
    pub fn missed_event_of_type(&self, event_type: &str) -> Option<&EventWithAttendance<'a>> {
        self.events.iter().find(|event| {
            &event.event.r#type == event_type
                && event.attendance.map(|a| a.deny_credit()).unwrap_or(false)
        })
    }

    pub fn events_of_type(
        &self,
        event_type: &str,
    ) -> impl Iterator<Item = &EventWithAttendance<'a>> {
        // TODO: use &str for event_type
        let event_type = event_type.to_owned();
        self.events
            .iter()
            .filter(move |event| &event.event.r#type == &event_type)
    }

    pub fn attended_volunteer_gig(&self, event: &EventWithAttendance<'a>) -> bool {
        if self.missed_event_of_type(EventType::REHEARSAL).is_some() {
            return false;
        }

        &event.event.r#type == EventType::VOLUNTEER_GIG
            && event.event.gig_count
            && event.did_attend()
    }

    pub fn is_bonus_event(&self, event: &EventWithAttendance<'a>) -> bool {
        let attended_first_sectional = self
            .events_of_type(EventType::SECTIONAL)
            .next()
            .map(|first_sectional| first_sectional.did_attend())
            .unwrap_or(false);

        &event.event.r#type == EventType::VOLUNTEER_GIG
            || &event.event.r#type == EventType::OMBUDS
            || (&event.event.r#type == EventType::OTHER && !event.should_attend())
            || (&event.event.r#type == EventType::SECTIONAL && attended_first_sectional)
    }
}
