use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use db::*;
use error::*;
use pinto::query_builder::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

impl Event {
    pub fn load<C: Connection>(event_id: i32, conn: &mut C) -> GreaseResult<EventWithGig> {
        conn.first(
            Select::new(Event::table_name())
                .join(Gig::table_name(), "id", "event", Join::Left)
                .fields(EventWithGigRow::field_names())
                .filter(&format!("id = {}", event_id)),
            format!("No event with id {}.", event_id),
        )
        .map(|row: EventWithGigRow| row.into())
    }

    pub fn load_all_for_current_semester<C: Connection>(
        conn: &mut C,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;

        conn.load_as::<EventWithGigRow, EventWithGig>(
            Select::new(Event::table_name())
                .join(Gig::table_name(), "id", "event", Join::Left)
                .fields(EventWithGigRow::field_names())
                .filter(&format!("semester = '{}'", &current_semester.name))
                .order_by("call_time", Order::Desc),
        )
    }

    pub fn load_all_of_type_for_current_semester<C: Connection>(
        event_type: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;

        conn.load_as::<EventWithGigRow, EventWithGig>(
            Select::new(Event::table_name())
                .join(Gig::table_name(), "id", "event", Join::Left)
                .fields(EventWithGigRow::field_names())
                .filter(&format!("semester = '{}'", &current_semester.name))
                .filter(&format!("type = '{}'", event_type))
                .order_by("call_time", Order::Desc),
        )
    }

    pub fn went_to_event_type_during_week_of(
        &self,
        semester_events_with_attendance: &Vec<(Event, Attendance)>,
        semester_absence_requests: &Vec<AbsenceRequest>,
        event_type: &str,
    ) -> Option<bool> {
        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);
        let now = Local::now().naive_local();

        let event_type_attendance_for_week = semester_events_with_attendance
            .iter()
            .filter(|(event, _attendance)| {
                event.id != self.id
                    && event.semester == self.semester
                    && event.call_time > last_sunday
                    && event.release_time.unwrap_or(event.call_time)
                        < std::cmp::min(next_sunday, now)
                    && event.type_ == event_type
            })
            .map(|(event, attendance)| (event.id, attendance.did_attend))
            .collect::<Vec<(i32, bool)>>();

        if event_type_attendance_for_week.len() == 0 {
            None
        } else {
            Some(
                event_type_attendance_for_week
                    .into_iter()
                    .any(|(event_id, did_attend)| {
                        did_attend
                            || semester_absence_requests.iter().any(|absence_request| {
                                absence_request.event == event_id
                                    && absence_request.state == AbsenceRequestState::Approved
                            })
                    }),
            )
        }
    }

    pub fn load_sectionals_the_week_of<C: Connection>(
        &self,
        conn: &mut C,
    ) -> GreaseResult<Vec<Event>> {
        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);

        conn.load(
            Select::new(Event::table_name())
                .fields(Event::field_names())
                .filter("type = 'sectional'")
                .filter(&format!("semester = '{}'", &self.semester))
                .filter(&format!("call_time > {}", to_value(last_sunday)))
                .filter(&format!("release_time > {}", to_value(next_sunday)))
                .order_by("call_time", Order::Asc),
        )
    }

    pub fn minimal(&self) -> Value {
        json!({
            "id": self.id,
            "name": &self.name,
        })
    }

    pub fn create(
        new_event: NewEvent,
        from_request: Option<(GigRequest, NewGig)>,
        conn: &mut DbConn,
    ) -> GreaseResult<i32> {
        if new_event.release_time.is_some()
            && new_event.release_time.unwrap() <= new_event.call_time
        {
            return Err(GreaseError::BadRequest(
                "release time must be after call time if it is supplied.".to_owned(),
            ));
        }

        let call_and_release_time_pairs = if let Some(period) = Period::parse(&new_event.repeat)? {
            let until = new_event.repeat_until.ok_or(GreaseError::BadRequest(
                "Must supply a repeat until time if repeat is supplied.".to_owned(),
            ))?;

            Event::repeat_event_times(new_event.call_time, new_event.release_time, period, until)
        } else {
            vec![(new_event.call_time, new_event.release_time)]
        };
        let num_events = call_and_release_time_pairs.len();
        if num_events == 0 {
            return Err(GreaseError::BadRequest(
                "the repeat setting would render no events, please check your repeat settings."
                    .to_owned(),
            ));
        }

        conn.transaction(|transaction| {
            let new_ids = call_and_release_time_pairs
                .iter()
                .map(|(call_time, release_time)| {
                    let new_id = transaction.insert_returning_id(
                        Insert::new(Event::table_name())
                            .set("name", &to_value(&new_event.name))
                            .set("semester", &to_value(&new_event.semester))
                            .set("`type`", &to_value(&new_event.type_))
                            .set("call_time", &to_value(&call_time))
                            .set("release_time", &to_value(&release_time))
                            .set("points", &to_value(&new_event.points))
                            .set("comments", &to_value(&new_event.comments))
                            .set("location", &to_value(&new_event.location))
                            .set("default_attend", &to_value(&new_event.default_attend))
                            .set("gig_count", &to_value(&new_event.gig_count)),
                    )?;
                    Attendance::create_for_new_event(new_id, transaction)?;

                    if let Some((ref _gig_request, ref new_gig)) = from_request.as_ref() {
                        Gig::insert(new_id, &new_gig, transaction)?;
                    }

                    Ok(new_id)
                })
                .collect::<GreaseResult<Vec<i32>>>()?;

            let first_id = *new_ids
                .iter()
                .nth(num_events - 1)
                .ok_or(GreaseError::ServerError(
                    "error inserting new event into database".to_owned(),
                ))?;

            if let Some((ref gig_request, ref _new_gig)) = from_request.as_ref() {
                transaction.update(
                    Update::new(GigRequest::table_name())
                        .filter(&format!("id = '{}'", &gig_request.id))
                        .set("event", &to_value(first_id))
                        .set("state", &to_value(GigRequestStatus::Accepted)),
                    format!(
                        "Error updating gig request with id {} to mark it as accepted.",
                        gig_request.id
                    ),
                )?;
            }

            Ok(first_id)
        })
    }

    pub fn repeat_event_times(
        call_time: NaiveDateTime,
        release_time: Option<NaiveDateTime>,
        period: Period,
        until: NaiveDate,
    ) -> Vec<(NaiveDateTime, Option<NaiveDateTime>)> {
        std::iter::successors(
            Some((call_time, release_time)),
            |(call_time, release_time)| {
                let duration = match period {
                    Period::Daily => Duration::days(1),
                    Period::Weekly => Duration::weeks(1),
                    Period::BiWeekly => Duration::weeks(2),
                    Period::Yearly => Duration::days(365),
                    Period::Monthly => {
                        let days = match call_time.month() {
                            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                            4 | 6 | 9 | 11 => 30,
                            // leap year check
                            2 => {
                                if NaiveDate::from_ymd_opt(call_time.year(), 2, 29).is_some() {
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
                    *call_time + duration,
                    release_time.as_ref().map(|time| *time + duration),
                ))
                .filter(|(call_time, _release_time)| call_time.date() < until)
            },
        )
        .collect::<Vec<_>>()
    }

    pub fn update(event_id: i32, event_update: EventUpdate, conn: &mut DbConn) -> GreaseResult<()> {
        let event = Event::load(event_id, conn)?;

        conn.transaction(move |transaction| {
            if let Some(_gig) = &event.gig {
                transaction.update(
                    Update::new(Gig::table_name())
                        .filter(&format!("event = {}", event_id))
                        .set(
                            "performance_time",
                            &to_value(
                                event_update
                                    .performance_time
                                    .ok_or(GreaseError::BadRequest(
                                        "Performance time is required on events that are gigs."
                                            .to_owned(),
                                    ))?,
                            ),
                        )
                        .set(
                            "uniform",
                            &to_value(event_update.uniform.ok_or(GreaseError::BadRequest(
                                "Uniform is required on events that are gigs.".to_owned(),
                            ))?),
                        )
                        .set("contact_name", &to_value(event_update.contact_name))
                        .set("contact_email", &to_value(event_update.contact_email))
                        .set("contact_phone", &to_value(event_update.contact_phone))
                        .set("price", &to_value(event_update.price))
                        .set(
                            "uniform",
                            &to_value(event_update.public.ok_or(GreaseError::BadRequest(
                                "Uniform is required on events that are gigs.".to_owned(),
                            ))?),
                        )
                        .set("summary", &to_value(event_update.summary))
                        .set("description", &to_value(event_update.description)),
                    format!("No event with id {}.", event_id),
                )?;
            } else if event_update.performance_time.is_some()
                || event_update.uniform.is_some()
                || event_update.contact_name.is_some()
                || event_update.contact_email.is_some()
                || event_update.contact_phone.is_some()
                || event_update.price.is_some()
                || event_update.public.is_some()
                || event_update.summary.is_some()
                || event_update.description.is_some()
            {
                transaction.insert(
                    Insert::new(Gig::table_name())
                        .set("event", &to_value(event_id))
                        .set(
                            "performance_time",
                            &to_value(
                                event_update
                                    .performance_time
                                    .ok_or(GreaseError::BadRequest(
                                        "Performance time is required on events that are gigs."
                                            .to_owned(),
                                    ))?,
                            ),
                        )
                        .set(
                            "uniform",
                            &to_value(event_update.uniform.ok_or(GreaseError::BadRequest(
                                "Uniform is required on events that are gigs.".to_owned(),
                            ))?),
                        )
                        .set("contact_name", &to_value(event_update.contact_name))
                        .set("contact_email", &to_value(event_update.contact_email))
                        .set("contact_phone", &to_value(event_update.contact_phone))
                        .set("price", &to_value(event_update.price))
                        .set(
                            "uniform",
                            &to_value(event_update.public.ok_or(GreaseError::BadRequest(
                                "Uniform is required on events that are gigs.".to_owned(),
                            ))?),
                        )
                        .set("summary", &to_value(event_update.summary))
                        .set("description", &to_value(event_update.description)),
                )?;
            }

            transaction.update(
                Update::new(Event::table_name())
                    .filter(&format!("id = {}", event_id))
                    .set("name", &to_value(event_update.name))
                    .set("semester", &to_value(event_update.semester))
                    .set("type", &to_value(event_update.type_))
                    .set("call_time", &to_value(event_update.call_time))
                    .set("release_time", &to_value(event_update.release_time))
                    .set("points", &to_value(event_update.points))
                    .set("comments", &to_value(event_update.comments))
                    .set("location", &to_value(event_update.location))
                    .set("gig_count", &to_value(event_update.gig_count))
                    .set("default_attend", &to_value(event_update.default_attend))
                    .set("section", &to_value(event_update.section)),
                format!("No event with id {}.", event_id),
            )
        })
    }

    pub fn delete<C: Connection>(event_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Event::table_name()).filter(&format!("id = {}", event_id)),
            format!("No event with id {}.", event_id),
        )
    }
}

impl Gig {
    pub fn insert<C: Connection>(
        event_id: i32,
        new_gig: &NewGig,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.insert(
            Insert::new(Event::table_name())
                .set("event", &to_value(event_id))
                .set("performance_time", &to_value(new_gig.performance_time))
                .set("uniform", &to_value(&new_gig.uniform))
                .set("contact_name", &to_value(&new_gig.contact_name))
                .set("contact_email", &to_value(&new_gig.contact_email))
                .set("contact_phone", &to_value(&new_gig.contact_phone))
                .set("price", &to_value(new_gig.price))
                .set("public", &to_value(new_gig.public))
                .set("summary", &to_value(&new_gig.summary))
                .set("description", &to_value(&new_gig.description)),
        )
    }
}

pub enum Period {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Yearly,
}

impl Period {
    pub fn parse(period: &str) -> GreaseResult<Option<Period>> {
        match period {
            "no" => Ok(None),
            "daily" => Ok(Some(Period::Daily)),
            "weekly" => Ok(Some(Period::Weekly)),
            "biweekly" => Ok(Some(Period::BiWeekly)),
            "monthly" => Ok(Some(Period::Monthly)),
            "yearly" => Ok(Some(Period::Yearly)),
            other => Err(GreaseError::BadRequest(format!(
                "The repeat value {} is not allowed. The only allowed values \
                 are 'no', 'daily', 'weekly', 'biweekly', 'monthly', or 'yearly'.",
                other
            ))),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EventWithGig {
    pub event: Event,
    pub gig: Option<Gig>,
}

impl EventWithGig {
    /// Render this event and gig's data to JSON.
    ///
    /// ## JSON Format:
    ///
    /// ```json
    /// {
    ///     id: integer,
    ///     name: string,
    ///     semester: string,
    ///     type: string,
    ///     callTime: datetime,
    ///     releaseTime: datetime, // optional
    ///     points: integer,
    ///     comments: string, // optional
    ///     location: string, // optional
    ///     gigCount: boolean,
    ///     defaultAttend: boolean,
    ///     section: string, // optional
    ///     performanceTime: datetime, // present for gigs
    ///     contactName: string, // optional for gigs
    ///     contactEmail: string, // optional for gigs
    ///     contactPhone: string, // optional for gigs
    ///     contactPhone: string, // optional for gigs
    ///     price: integer, // optional for gigs
    ///     public: boolean, // present for gigs
    ///     summary: string, // optional for gigs
    ///     description: string, // optional for gigs
    ///     uniform: integer // present for gigs
    /// }
    /// ```
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.event.id,
            "name": self.event.name,
            "semester": self.event.semester,
            "type": self.event.type_,
            "callTime": self.event.call_time,
            "releaseTime": self.event.release_time,
            "points": self.event.points,
            "comments": self.event.comments,
            "location": self.event.location,
            "gig_count": self.event.gig_count,
            "defaultAttend": self.event.default_attend,
            "section": self.event.section,
            "performanceTime": self.gig.as_ref().map(|gig| gig.performance_time),
            "uniform": self.gig.as_ref().map(|gig| gig.uniform),
            "contactName": self.gig.as_ref().map(|gig| &gig.contact_name),
            "contactEmail": self.gig.as_ref().map(|gig| &gig.contact_email),
            "contactPhone": self.gig.as_ref().map(|gig| &gig.contact_phone),
            "price": self.gig.as_ref().map(|gig| gig.price),
            "public": self.gig.as_ref().map(|gig| gig.public),
            "summary": self.gig.as_ref().map(|gig| &gig.summary),
            "description": self.gig.as_ref().map(|gig| &gig.description),
        })
    }

    /// Render this event and gig's data to JSON, including some additional data.
    ///
    /// On top of what is included for [to_json](#method.to_json), two other fields are included:
    ///   * uniform: if the event has a gig, the gig's [Uniform](../struct.Uniform.html) is included
    ///       in place of the uniform's id. It is null otherwise.
    ///   * attendance: The current member's [Attendance](../struct.Attendance.html) for the gig is
    ///       included if they were ever active during the semester of the event. It is null otherwise.
    pub fn to_json_full<C: Connection>(
        &self,
        member: &Member,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();

        let uniform = if let Some(uniform) = self.gig.as_ref().map(|gig| gig.uniform) {
            Some(Uniform::load(uniform, conn)?)
        } else {
            None
        };
        json_val["uniform"] = json!(uniform);

        let attendance = Attendance::load(&member.email, self.event.id, conn)?;
        json_val["attendance"] = json!(attendance);

        Ok(json_val)
    }
}

impl GigRequest {
    pub fn load<C: Connection>(id: i32, conn: &mut C) -> GreaseResult<GigRequest> {
        conn.first(
            &GigRequest::filter(&format!("id = {}", id)),
            format!("no gig request with id {}", id),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<GigRequest>> {
        conn.load(&GigRequest::select_all_in_order("time", Order::Desc))
    }

    pub fn load_all_for_semester_and_pending<C: Connection>(
        conn: &mut C,
    ) -> GreaseResult<Vec<GigRequest>> {
        let current_semester = Semester::load_current(conn)?;

        conn.load(
            GigRequest::filter(&format!(
                "time > {} OR status = '{}'",
                &current_semester.start_date.to_value().as_sql(false),
                GigRequestStatus::Pending
            ))
            .order_by("time", Order::Desc),
        )
    }

    pub fn set_status<C: Connection>(
        request_id: i32,
        status: GigRequestStatus,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use self::GigRequestStatus::*;
        let request = GigRequest::load(request_id, conn)?;

        match (&request.status, &status) {
            (Pending, Pending) | (Dismissed, Dismissed) | (Accepted, Accepted) => Ok(()),
            (Accepted, _other) => Err(GreaseError::BadRequest(
                "Cannot change the status of an accepted gig request.".to_owned(),
            )),
            (Dismissed, Accepted) => Err(GreaseError::BadRequest(
                "Cannot directly accept a gig request if it is dismissed. Please reopen it first."
                    .to_owned(),
            )),
            _allowed_change => {
                if request.status == Pending && status == Accepted && request.event.is_none() {
                    Err(GreaseError::BadRequest("Must create the event for the gig request first before marking it as accepted.".to_owned()))
                } else {
                    conn.update_opt(
                        Update::new(Event::table_name())
                            .filter(&format!("id = {}", request_id))
                            .set("status", &format!("'{}'", status)),
                    )
                }
            }
        }
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
struct EventWithGigRow {
    // event fields
    pub id: i32,
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<String>,
    // gig fields
    pub event: Option<i32>,
    pub performance_time: Option<NaiveDateTime>,
    pub uniform: Option<i32>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: Option<bool>,
    pub summary: Option<String>,
    pub description: Option<String>,
}

impl Into<EventWithGig> for EventWithGigRow {
    fn into(self) -> EventWithGig {
        EventWithGig {
            event: Event {
                id: self.id,
                name: self.name,
                semester: self.semester,
                type_: self.type_,
                call_time: self.call_time,
                release_time: self.release_time,
                points: self.points,
                comments: self.comments,
                location: self.location,
                gig_count: self.gig_count,
                default_attend: self.default_attend,
                section: self.section,
            },
            gig: if self.event.is_some()
                && self.performance_time.is_some()
                && self.uniform.is_some()
                && self.public.is_some()
            {
                Some(Gig {
                    event: self.event.unwrap(),
                    performance_time: self.performance_time.unwrap(),
                    uniform: self.uniform.unwrap(),
                    contact_name: self.contact_name,
                    contact_email: self.contact_email,
                    contact_phone: self.contact_phone,
                    price: self.price,
                    public: self.public.unwrap(),
                    summary: self.summary,
                    description: self.description,
                })
            } else {
                None
            },
        }
    }
}
