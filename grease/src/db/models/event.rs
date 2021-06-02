use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use db::models::member::MemberForSemester;
use db::schema::{attendance, event, gig, gig_request, AbsenceRequestState, GigRequestStatus};
use db::{
    AbsenceRequest, Attendance, Event, EventUpdate, Gig, GigRequest, NewEvent, NewEventFields,
    NewGig, Period, Semester, Uniform,
};
use diesel::prelude::*;
use error::*;
use icalendar::{Calendar, Component, Event as CalEvent, Property};
use serde::Serialize;
use std::iter::FromIterator;

impl Event {
    pub const REHEARSAL: &'static str = "Rehearsal";
    pub const SECTIONAL: &'static str = "Sectional";
    pub const VOLUNTEER_GIG: &'static str = "Volunteer Gig";
    pub const TUTTI_GIG: &'static str = "Tutti Gig";
    pub const OMBUDS: &'static str = "Ombuds";
    pub const OTHER: &'static str = "Other";

    pub fn is_gig(&self) -> bool {
        self.type_ == Self::VOLUNTEER_GIG || self.type_ == Self::TUTTI_GIG
    }

    pub fn load(event_id: i32, conn: &MysqlConnection) -> GreaseResult<EventWithGig> {
        event::table
            .left_outer_join(gig::table)
            .filter(event::id.eq(event_id))
            .first::<(Event, Option<Gig>)>(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .map(|(e, g)| EventWithGig { event: e, gig: g })
            .ok_or(GreaseError::BadRequest(format!(
                "No event with id {}.",
                event_id
            )))
    }

    pub fn load_all_for_current_semester(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;

        let rows = event::table
            .left_outer_join(gig::table)
            .filter(event::semester.eq(current_semester.name))
            .order_by(event::call_time.asc())
            .load::<(Event, Option<Gig>)>(conn)?;

        Ok(rows
            .into_iter()
            .map(|(e, g)| EventWithGig { event: e, gig: g })
            .collect())
    }

    pub fn load_all_of_type_for_current_semester(
        event_type: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;

        let rows = event::table
            .left_outer_join(gig::table)
            .filter(
                event::semester
                    .eq(current_semester.name)
                    .and(event::type_.eq(event_type)),
            )
            .order_by(event::call_time.asc())
            .load::<(Event, Option<Gig>)>(conn)?;

        Ok(rows
            .into_iter()
            .map(|(e, g)| EventWithGig { event: e, gig: g })
            .collect())
    }

    pub fn load_all_public_events_for_current_semester(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<PublicEvent>> {
        let current_semester = Semester::load_current(conn)?;
        let rows = event::table
            .inner_join(gig::table)
            .filter(
                event::semester
                    .eq(&current_semester.name)
                    .and(gig::public.eq(true)),
            )
            .order_by(gig::performance_time)
            .load::<(Event, Gig)>(conn)?;

        rows.into_iter()
            .map(|(e, g)| EventWithGig::to_public(e, g))
            .collect()
    }

    pub fn went_to_event_type_during_week_of(
        &self,
        events_with_attendance: &Vec<(EventWithGig, Attendance)>,
        absence_requests: &Vec<AbsenceRequest>,
        event_type: &str,
    ) -> Option<bool> {
        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);
        let now = Local::now().naive_local();

        let event_type_attendance_for_week = events_with_attendance
            .iter()
            .filter(|(given_event, _attendance)| {
                let event_end_time = given_event
                    .event
                    .release_time
                    .unwrap_or(given_event.event.call_time);
                given_event.event.id != self.id
                    && given_event.event.semester == self.semester
                    && given_event.event.call_time > last_sunday
                    && event_end_time < std::cmp::min(next_sunday, now)
                    && given_event.event.type_ == event_type
            })
            .map(|(given_event, attendance)| (given_event.event.id, attendance))
            .collect::<Vec<(i32, &Attendance)>>();

        let number_of_events_to_ignore = event_type_attendance_for_week
            .iter()
            .filter(|(_event_id, attendance)| !attendance.should_attend && !attendance.did_attend)
            .count();

        if event_type_attendance_for_week.len() == number_of_events_to_ignore {
            None
        } else {
            Some(
                event_type_attendance_for_week
                    .into_iter()
                    .any(|(event_id, attendance)| {
                        attendance.did_attend
                            || absence_requests.iter().any(|absence_request| {
                                absence_request.event == event_id
                                    && absence_request.state == AbsenceRequestState::Approved
                            })
                    }),
            )
        }
    }

    pub fn load_sectionals_the_week_of(&self, conn: &MysqlConnection) -> GreaseResult<Vec<Event>> {
        use db::schema::event::dsl::*;

        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);

        event
            .filter(
                type_
                    .eq("sectional")
                    .and(semester.eq(&self.semester))
                    .and(call_time.gt(last_sunday))
                    .and(release_time.gt(next_sunday)),
            )
            .order_by(call_time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(
        new_event: NewEvent,
        from_request: Option<GigRequest>,
        conn: &MysqlConnection,
    ) -> GreaseResult<i32> {
        use db::schema::event::dsl::*;

        let new_gig = if let Some(ref request) = from_request {
            if let Some(new_gig) = new_event.gig {
                Some(new_gig)
            } else {
                Some(request.build_new_gig(conn)?)
            }
        } else {
            new_event.gig
        };

        if new_event.fields.release_time.is_some()
            && new_event.fields.release_time.unwrap() <= new_event.fields.call_time
        {
            return Err(GreaseError::BadRequest(
                "Release time must be after call time if it is supplied.".to_owned(),
            ));
        }

        let until = if new_event.repeat == Period::No {
            new_event.fields.call_time.date()
        } else {
            new_event.repeat_until.ok_or(GreaseError::BadRequest(
                "Must supply a repeat until time if repeat is supplied.".to_owned(),
            ))?
        };
        let call_and_release_time_pairs = Event::repeat_event_times(
            &new_event.fields.call_time,
            &new_event.fields.release_time,
            new_event.repeat,
            until,
        );

        let num_events = call_and_release_time_pairs.len();
        if num_events == 0 {
            return Err(GreaseError::BadRequest(
                "The repeat setting would render no events, please check your repeat settings."
                    .to_owned(),
            ));
        }

        let event_fields = new_event.fields;
        conn.transaction(|| {
            diesel::insert_into(event)
                .values(
                    &call_and_release_time_pairs
                        .into_iter()
                        .map(|(new_call_time, new_release_time)| NewEventFields {
                            call_time: new_call_time,
                            release_time: new_release_time,
                            ..event_fields.clone()
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(conn)?;

            let new_ids = event
                .select(id)
                .order_by(id.desc())
                .limit(num_events as i64)
                .load(conn)?;
            new_ids
                .iter()
                .map(|&new_id| Attendance::create_for_new_event(new_id, conn))
                .collect::<GreaseResult<_>>()?;
            if let Some(new_gig) = new_gig {
                diesel::insert_into(gig::table)
                    .values(
                        &new_ids
                            .iter()
                            .map(|new_id| new_gig.clone().as_full_gig(*new_id))
                            .collect::<Vec<_>>(),
                    )
                    .execute(conn)?;
            }

            let first_id = *new_ids
                .iter()
                .nth(num_events - 1)
                .ok_or(GreaseError::ServerError(
                    "error inserting new event into database".to_owned(),
                ))?;

            if let Some(ref from_request) = from_request.as_ref() {
                diesel::update(gig_request::table.filter(gig_request::id.eq(&from_request.id)))
                    .set((
                        gig_request::event.eq(first_id),
                        gig_request::status.eq(GigRequestStatus::Accepted),
                    ))
                    .execute(conn)?;

                // format!(
                //     "Error updating gig request with id {} to mark it as accepted.",
                //     gig_request.id
                // ),
            }

            Ok(first_id)
        })
    }

    pub fn repeat_event_times(
        given_call_time: &NaiveDateTime,
        given_release_time: &Option<NaiveDateTime>,
        period: Period,
        until: NaiveDate,
    ) -> Vec<(NaiveDateTime, Option<NaiveDateTime>)> {
        std::iter::successors(
            Some((given_call_time.clone(), given_release_time.clone())),
            |(given_call_time, given_release_time)| {
                let duration = match period {
                    Period::No => return None,
                    Period::Daily => Duration::days(1),
                    Period::Weekly => Duration::weeks(1),
                    Period::Biweekly => Duration::weeks(2),
                    Period::Yearly => Duration::days(365),
                    Period::Monthly => {
                        let days = match given_call_time.month() {
                            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                            4 | 6 | 9 | 11 => 30,
                            // leap year check
                            2 => {
                                if NaiveDate::from_ymd_opt(given_call_time.year(), 2, 29).is_some()
                                {
                                    29
                                } else {
                                    28
                                }
                            }
                            _ => unreachable!(),
                        };
                        Duration::days(days)
                    }
                };

                Some((
                    *given_call_time + duration,
                    given_release_time.as_ref().map(|time| *time + duration),
                ))
                .filter(|(given_call_time, _release_time)| given_call_time.date() < until)
            },
        )
        .collect::<Vec<_>>()
    }

    pub fn update(
        event_id: i32,
        event_update: EventUpdate,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let found_event = Event::load(event_id, conn)?;

        if found_event.gig.is_some() && event_update.gig.is_none() {
            return Err(GreaseError::BadRequest(
                "Gig fields must be present when updating gig events.".to_owned(),
            ));
        }

        conn.transaction(|| {
            diesel::update(event::table.filter(event::id.eq(event_id)))
                .set(&event_update.fields)
                .execute(conn)?;

            if found_event.gig.is_some() {
                if let Some(gig_update) = event_update.gig {
                    diesel::update(gig::table.filter(gig::event.eq(event_id)))
                        .set(&gig_update)
                        .execute(conn)?;
                }
            }

            Ok(())
        })
        .map_err(GreaseError::DbError)
    }

    pub fn rsvp(
        event_id: i32,
        member: &MemberForSemester,
        attending: bool,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let found_event = Event::load(event_id, conn)?;
        let attendance = Attendance::load(&member.member.email, event_id, conn)?.ok_or(
            GreaseError::ServerError(format!(
                "No attendance exists for member {} at event with id {}.",
                &member.member.email, event_id,
            )),
        )?;
        Self::check_for_rsvp_issue(
            &found_event.event,
            &attendance,
            member.active_semester.is_some(),
        )?;

        diesel::update(
            attendance::table.filter(
                attendance::event
                    .eq(event_id)
                    .and(attendance::member.eq(&member.member.email)),
            ),
        )
        .set((
            attendance::should_attend.eq(attending),
            attendance::confirmed.eq(true),
        ))
        .execute(conn)?;

        Ok(())
    }

    pub fn confirm(
        event_id: i32,
        member: &MemberForSemester,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let _found_event = Event::load(event_id, conn)?;
        let _attendance = Attendance::load(&member.member.email, event_id, conn)?.ok_or(
            GreaseError::ServerError(format!(
                "No attendance exists for member {} at event with id {}.",
                &member.member.email, event_id,
            )),
        )?;

        diesel::update(
            attendance::table.filter(
                attendance::event
                    .eq(event_id)
                    .and(attendance::member.eq(&member.member.email)),
            ),
        )
        .set((
            attendance::should_attend.eq(true),
            attendance::confirmed.eq(true),
        ))
        .execute(conn)?;

        Ok(())
    }

    pub fn delete(event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        diesel::delete(event::table.filter(event::id.eq(event_id))).execute(conn)?;

        Ok(())
        // format!("No event with id {}.", event_id),
    }

    pub fn check_for_rsvp_issue(
        given_event: &Event,
        given_attendance: &Attendance,
        is_active: bool,
    ) -> GreaseResult<()> {
        match Self::rsvp_issue(given_event, Some(given_attendance), is_active) {
            Some(issue) => Err(GreaseError::BadRequest(issue)),
            None => Ok(()),
        }
    }

    pub fn rsvp_issue(
        &self,
        given_attendance: Option<&Attendance>,
        is_active: bool,
    ) -> Option<String> {
        if !is_active {
            Some("Member must be active to RSVP to events.".to_owned())
        } else if !given_attendance.map(|a| a.should_attend).unwrap_or(true) {
            None
        } else if Local::now().naive_local() + Duration::days(1) > self.call_time {
            Some("Responses are closed for this event.".to_owned())
        } else if let Some(bad_type) = ["Tutti Gig", "Sectional", "Rehearsal"]
            .iter()
            .find(|t| t == &&self.type_)
        {
            Some(format!("You cannot RSVP for {} events.", bad_type))
        } else {
            None
        }
    }
}

#[derive(Serialize)]
pub struct PublicEvent {
    pub id: i32,
    pub name: String,
    pub time: i64,
    pub location: String,
    pub summary: String,
    pub description: String,
    pub invite: String,
}

#[derive(Serialize)]
pub struct EventWithGig {
    #[serde(flatten)]
    pub event: Event,
    pub gig: Option<Gig>,
}

impl EventWithGig {
    fn build_calendar_event(event: &Event, gig: &Gig) -> CalEvent {
        let end_time = event
            .release_time
            .unwrap_or(gig.performance_time + Duration::hours(1));
        let location = event.location.clone().unwrap_or_default();

        CalEvent::new()
            .summary(&event.name)
            .description(&gig.summary.clone().unwrap_or_default())
            .starts(Utc.from_local_datetime(&gig.performance_time).unwrap())
            .ends(Utc.from_local_datetime(&end_time).unwrap())
            .append_property(Property::new("LOCATION", &location).done())
            .done()
    }

    fn build_calendar(event: &Event, gig: &Gig) -> Calendar {
        let calendar_event = Self::build_calendar_event(event, gig);

        Calendar::from_iter(vec![calendar_event])
    }

    fn build_calendar_url(event: &Event, gig: &Gig) -> String {
        let calendar = Self::build_calendar(event, gig);
        let encoded_calendar = base64::encode_config(&calendar.to_string(), base64::URL_SAFE);

        format!("data:text/calendar;base64,{}", encoded_calendar)
    }

    pub fn to_public(event: Event, gig: Gig) -> GreaseResult<PublicEvent> {
        if !gig.public {
            return Err(GreaseError::BadRequest(format!(
                "Event with id {} is not a public event.",
                event.id
            )));
        }

        let calendar_url = Self::build_calendar_url(&event, &gig);
        Ok(PublicEvent {
            id: event.id,
            name: event.name,
            time: gig.performance_time.timestamp() * 1000,
            location: event.location.unwrap_or_default(),
            summary: gig.summary.unwrap_or_default(),
            description: gig.description.unwrap_or_default(),
            invite: calendar_url,
        })
    }
}

impl GigRequest {
    pub fn load(given_id: i32, conn: &MysqlConnection) -> GreaseResult<GigRequest> {
        gig_request::table
            .filter(gig_request::id.eq(given_id))
            .first(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "no gig request with id {}",
                given_id
            )))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<GigRequest>> {
        gig_request::table
            .order_by(gig_request::time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_semester_and_pending(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<GigRequest>> {
        let current_semester = Semester::load_current(conn)?;
        gig_request::table
            .filter(
                gig_request::time
                    .gt(current_semester.start_date)
                    .or(gig_request::status.eq(GigRequestStatus::Pending)),
            )
            .order_by(gig_request::time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn build_new_gig(&self, conn: &MysqlConnection) -> GreaseResult<NewGig> {
        Ok(NewGig {
            performance_time: self.start_time,
            uniform: Uniform::load_default(conn)?.id,
            contact_name: Some(self.contact_name.clone()),
            contact_email: Some(self.contact_email.clone()),
            contact_phone: Some(self.contact_phone.clone()),
            price: None,
            public: false,
            summary: None,
            description: None,
        })
    }

    pub fn set_status(
        request_id: i32,
        given_status: GigRequestStatus,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use self::GigRequestStatus::*;
        let request = GigRequest::load(request_id, conn)?;

        match (&request.status, &given_status) {
            (Pending, Pending) | (Dismissed, Dismissed) | (Accepted, Accepted) => Ok(()),
            (Accepted, _other) => Err(GreaseError::BadRequest(
                "Cannot change the status of an accepted gig request.".to_owned(),
            )),
            (Dismissed, Accepted) => Err(GreaseError::BadRequest(
                "Cannot directly accept a gig request if it is dismissed. Please reopen it first."
                    .to_owned(),
            )),
            _allowed_change => {
                if request.status == Pending && given_status == Accepted && request.event.is_none()
                {
                    Err(GreaseError::BadRequest("Must create the event for the gig request first before marking it as accepted.".to_owned()))
                } else {
                    diesel::update(gig_request::table.filter(gig_request::id.eq(request_id)))
                        .set(gig_request::status.eq(given_status))
                        .execute(conn)?;

                    Ok(())
                }
            }
        }
    }
}

impl NewGig {
    pub fn as_full_gig(self, event_id: i32) -> Gig {
        Gig {
            event: event_id,
            performance_time: self.performance_time,
            uniform: self.uniform,
            contact_name: self.contact_name,
            contact_email: self.contact_email,
            contact_phone: self.contact_phone,
            price: self.price,
            public: self.public,
            summary: self.summary,
            description: self.description,
        }
    }
}
