use crate::models::member::grades::context::AttendanceContext;
use crate::models::event::absence_request::AbsenceRequest;

pub struct EventWithAttendance<'a> {
    pub event: &'a Event,
    pub gig: Option<&'a Gig>,
    pub attendance: Option<&'a AttendanceContext>,
}

impl<'a> EventWithAttendance<'a> {
    pub fn is_bonus_event(&self, attended_first_sectional: bool) -> bool {
      self.event.r#type == Event::VOLUNTEER_GIG || self.event.r#type == Event::OMBUDS ||
        (self.event.r#type == Event::OTHER && !self.attendance.should_attend) ||
        (self.event.r#type == Event::SECTIONAL && attended_first_sectional)
    }
}

pub struct WeekOfAttendances<'a> {
    pub events: Vec<EventWithAttendance<'a>>,
}

impl<'a> WeekOfAttendances<'a> {
    pub fn missed_event_of_type(&self, event_type: &str) -> Option<EventWithAttendance<'a>> {
        self.events.iter().filter(|event| event.event.r#type == event_type && event.attendance.map(|a| a.deny_credit()).unwrap_or(false))
    }

    // pub fn attended_event_of_type(&self, event_type: &str) -> Option<EventWithAttendance<'a>> {
    //     self.events.iter().filter(|event| event.event.r#type == event_type && event.attendance.map(|a| a.deny_credit()).unwrap_or(false))
    // }

    pub fn events_of_type(&self, event_type: &str) -> impl Iterator<Item = EventWithAttendance<'a>> {
        self.events.iter().filter(|event| event.event.r#type == event_type)
    }

    pub fn attended_volunteer_gig(&self, event: EventWithAttendance<'a>) -> bool {
        if self.missed_event_of_type(Event::REHEARSAL) {
            return None;
        }

        event.event.r#type == Event::VOLUNTEER_GIG && event.event.gig_count && event.attendance.map(|a| a.did_attend).unwrap_or(false)
    }
}
