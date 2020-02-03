use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use db::models::grades::GradeChange;
use db::models::member::MemberForSemester;
use db::schema::{
    attendance, event::dsl::*, gig, gig_request, AbsenceRequestState, GigRequestStatus,
};
use db::{
    AbsenceRequest, Attendance, Event, EventUpdate, Gig, GigRequest, NewEvent, NewEventFields,
    NewGig, Semester, Uniform,
};
use diesel::prelude::*;
use error::*;
use serde::Serialize;
use serde_json::{json, Value};

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
        use db::schema::event::dsl::*;

        event
            .left_outer_join(gig::table)
            .filter(id.eq(event_id))
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
        use db::schema::event::dsl::*;

        let current_semester = Semester::load_current(conn)?;

        event
            .left_outer_join(gig::table)
            .filter(semester.eq(current_semester.name))
            .order_by(call_time.asc())
            .load::<(Event, Option<Gig>)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(e, g)| EventWithGig { event: e, gig: g })
                    .collect()
            })
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_of_type_for_current_semester(
        event_type: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventWithGig>> {
        use db::schema::event::dsl::*;

        let current_semester = Semester::load_current(conn)?;

        event
            .left_outer_join(gig::table)
            .filter(semester.eq(current_semester.name).and(type_.eq(event_type)))
            .order_by(call_time.asc())
            .load::<(Event, Option<Gig>)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(e, g)| EventWithGig { event: e, gig: g })
                    .collect()
            })
            .map_err(GreaseError::DbError)
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

        let period = new_event.repeat.parse::<Period>()?;
        let until = if period == Period::No {
            new_event.fields.call_time.date()
        } else {
            new_event.repeat_until.ok_or(GreaseError::BadRequest(
                "Must supply a repeat until time if repeat is supplied.".to_owned(),
            ))?
        };
        let call_and_release_time_pairs = Event::repeat_event_times(
            &new_event.fields.call_time,
            &new_event.fields.release_time,
            period,
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
                    Period::BiWeekly => Duration::weeks(2),
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
            diesel::update(event.filter(id.eq(event_id)))
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
        diesel::delete(event.filter(id.eq(event_id))).execute(conn)?;

        Ok(())
        // format!("No event with id {}.", event_id),
    }

    pub fn check_for_rsvp_issue(
        given_event: &Event,
        given_attendance: &Attendance,
        is_active: bool,
    ) -> GreaseResult<()> {
        match Self::rsvp_issue(given_event, given_attendance, is_active) {
            Some(issue) => Err(GreaseError::BadRequest(issue)),
            None => Ok(()),
        }
    }

    pub fn rsvp_issue(
        given_event: &Event,
        given_attendance: &Attendance,
        is_active: bool,
    ) -> Option<String> {
        if !is_active {
            Some("Member must be active to RSVP to events.".to_owned())
        } else if !given_attendance.should_attend {
            None
        } else if Local::now().naive_local() + Duration::days(1) > given_event.call_time {
            Some("Responses are closed for this event.".to_owned())
        } else if let Some(bad_type) = ["Tutti Gig", "Sectional", "Rehearsal"]
            .iter()
            .find(|given_event_type| given_event_type == &&given_event.type_)
        {
            Some(format!("You cannot RSVP for {} events.", bad_type))
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
pub enum Period {
    No,
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Yearly,
}

impl std::str::FromStr for Period {
    type Err = GreaseError;

    fn from_str(period: &str) -> GreaseResult<Period> {
        match period {
            "no" => Ok(Period::No),
            "daily" => Ok(Period::Daily),
            "weekly" => Ok(Period::Weekly),
            "biweekly" => Ok(Period::BiWeekly),
            "monthly" => Ok(Period::Monthly),
            "yearly" => Ok(Period::Yearly),
            other => Err(GreaseError::BadRequest(format!(
                "The repeat value {} is not allowed. The only allowed values \
                 are 'no', 'daily', 'weekly', 'biweekly', 'monthly', or 'yearly'.",
                other
            ))),
        }
    }
}

#[derive(Serialize)]
pub struct EventWithGig {
    #[serde(flatten)]
    pub event: Event,
    #[serde(flatten)]
    pub gig: Option<Gig>,
}

impl EventWithGig {
    /// Render this event and gig's data to JSON, including some additional data.
    ///
    /// On top of what is included for [to_json](#method.to_json), two other fields are included:
    ///   * attendance: The current member's [Attendance](../struct.Attendance.html) for the gig is
    ///       included if they were ever active during the semester of the event. It is null otherwise.
    pub fn to_json_with_grade_change<'e>(
        &'e self,
        grade_change: Option<&GradeChange>,
        is_active: bool,
    ) -> Value {
        #[derive(Serialize)]
        struct JsonFormat<'e> {
            #[serde(flatten)]
            event: &'e EventWithGig,
            #[serde(rename = "gradeChange")]
            grade_change: Option<GradeChangeFormat<'e>>,
            attendance: Option<&'e Attendance>,
            #[serde(rename = "absenceRequest")]
            absence_request: Option<&'e AbsenceRequest>,
            #[serde(rename = "rsvpIssue")]
            rsvp_issue: Option<String>,
        }

        #[derive(Serialize)]
        struct GradeChangeFormat<'g> {
            pub reason: &'g str,
            pub change: f32,
            #[serde(rename = "partialScore")]
            pub partial_score: f32,
        }

        if let Some(ref change) = grade_change {
            json!(JsonFormat {
                event: self,
                attendance: Some(&change.attendance),
                absence_request: change.absence_request.as_ref(),
                grade_change: Some(GradeChangeFormat {
                    reason: &change.reason,
                    change: change.change,
                    partial_score: change.partial_score,
                }),
                rsvp_issue: Event::rsvp_issue(&self.event, &change.attendance, is_active),
            })
        } else {
            json!(JsonFormat {
                event: self,
                attendance: None,
                absence_request: None,
                grade_change: None,
                rsvp_issue: Some("Inactive members cannot RSVP for events.".to_owned()),
            })
        }
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
