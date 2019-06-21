use chrono::Datelike;
use chrono::{Duration, Local};
use db::models::*;
use db::traits::*;
use error::*;
use mysql::{prelude::{GenericConnection, ToValue}, Conn};
use pinto::query_builder::{self, Join, Order};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

impl Event {
    pub fn load<G: GenericConnection>(given_event_id: i32, conn: &mut G) -> GreaseResult<EventWithGig> {
        let query = query_builder::select(Self::table_name())
            .join(Gig::table_name(), "id", "event", Join::Left)
            .fields(EventWithGigRow::field_names())
            .filter(&format!("id = {}", given_event_id))
            .build();

        match conn
            .first::<_, EventWithGigRow>(query)
            .map_err(GreaseError::DbError)?
        {
            Some(row) => Ok(row.into()),
            None => Err(GreaseError::BadRequest(format!(
                "event with id {} doesn't exist",
                given_event_id
            ))),
        }
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<EventWithGig>> {
        let query = query_builder::select(Self::table_name())
            .join(Gig::table_name(), "id", "event", Join::Left)
            .fields(EventWithGigRow::field_names())
            .order_by("call_time", Order::Desc)
            .build();

        crate::db::load::<EventWithGigRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
    }

    pub fn load_all_for_current_semester<G: GenericConnection>(
        conn: &mut G,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;
        let query = query_builder::select(Self::table_name())
            .join(Gig::table_name(), "id", "event", Join::Left)
            .fields(EventWithGigRow::field_names())
            .filter(&format!("semester = '{}'", &current_semester.name))
            .order_by("call_time", Order::Desc)
            .build();

        crate::db::load::<EventWithGigRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
    }

    pub fn load_all_for_semester_until_now(
        semester: &str,
        conn: &mut Conn,
    ) -> GreaseResult<Vec<Event>> {
        let now = Local::now().naive_local();
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!(
                "semester = '{}' AND release_time < {}",
                semester,
                now.to_value().as_sql(false)
            ))
            .order_by("call_time", Order::Asc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn load_all_of_type_for_current_semester(
        given_event_type: &str,
        conn: &mut Conn,
    ) -> GreaseResult<Vec<EventWithGig>> {
        let current_semester = Semester::load_current(conn)?;
        let query = query_builder::select(Self::table_name())
            .join(Gig::table_name(), "id", "event", Join::Left)
            .fields(EventWithGigRow::field_names())
            .filter(&format!(
                "semester = '{}' AND type = '{}'",
                &current_semester.name, given_event_type
            ))
            .order_by("call_time", Order::Desc)
            .build();

        crate::db::load::<EventWithGigRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
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

    // TODO: fix for weekend sectionals?
    pub fn load_sectionals_the_week_of(&self, conn: &mut Conn) -> GreaseResult<Vec<Event>> {
        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);

        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter("type = 'sectional'")
            .filter(&format!("semester = '{}'", &self.semester))
            .filter(&format!(
                "call_time > {}",
                last_sunday.to_value().as_sql(true)
            ))
            .filter(&format!(
                "release_time < {}",
                next_sunday.to_value().as_sql(true)
            ))
            .order_by("call_time", Order::Asc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn minimal(&self) -> Value {
        json!({
            "id": self.id,
            "name": &self.name,
        })
    }

    pub fn create(new_event: NewEvent, conn: &mut Conn) -> GreaseResult<i32> {
        if new_event.release_time.is_some() && new_event.release_time.unwrap() <= new_event.call_time {
            return Err(GreaseError::BadRequest("release time must be after call time if it is supplied.".to_owned()));
        }

        let call_and_release_time_pairs = if &new_event.repeat == "no" {
            vec![(new_event.call_time, new_event.release_time)]
        } else {
            enum Period {
                Daily,
                Weekly,
                BiWeekly,
                Monthly,
                Yearly,
            }
            let given_duration = match new_event.repeat.as_str() {
                "daily" => Period::Daily,
                "weekly" => Period::Weekly,
                "biweekly" => Period::BiWeekly,
                "monthly" => Period::Monthly,
                "yearly" => Period::Yearly,
                other => return Err(GreaseError::BadRequest(format!(
                    "The repeat value '{}' is not allowed. The only allowed values \
                     are 'no', 'daily', 'weekly', 'biweekly', 'monthly', or 'yearly'.", other))),
            };
            let until = new_event.repeat_until.ok_or(GreaseError::BadRequest(
                "Must supply a repeat until time if repeat is supplied.".to_owned()))?;

            std::iter::successors(Some((new_event.call_time, new_event.release_time)), |(call_time, release_time)| {
                let duration = match given_duration {
                    Period::Daily => Duration::days(1),
                    Period::Weekly => Duration::weeks(1),
                    Period::BiWeekly => Duration::weeks(2),
                    Period::Yearly => Duration::days(365),
                    Period::Monthly => {
                        let days = match call_time.month() {
                            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                            4 | 6 | 9 | 11 => 30,
                            2 => if NaiveDate::from_ymd_opt(call_time.year(), 2, 29).is_some() {
                                29
                            } else {
                                28
                            },
                            _ => unreachable!(),
                        };
                        Duration::days(days)
                    }
                };

                Some((*call_time + duration, release_time.as_ref().map(|time| *time + duration))).filter(|(call_time, _release_time)| call_time.date() < until)
            })
            .collect::<Vec<_>>()
        };
        let num_events = call_and_release_time_pairs.len();
        if num_events == 0 {
            return Err(GreaseError::BadRequest("the repeat setting would render no events, please check your repeat settings.".to_owned()));
        }

        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;

        for (call_time, release_time) in call_and_release_time_pairs {
            let query = query_builder::insert(Self::table_name())
                .set("name", &format!("'{}'", &new_event.name))
                .set("semester", &format!("'{}'", &new_event.semester))
                .set("`type`", &format!("'{}'", &new_event.type_))
                .set("call_time", &call_time.to_value().as_sql(false))
                .set(
                    "release_time",
                    &release_time.to_value().as_sql(false),
                )
                .set("points", &new_event.points.to_string())
                .set("comments", &new_event.comments.to_value().as_sql(false))
                .set("location", &new_event.location.to_value().as_sql(false))
                .set("default_attend", &new_event.default_attend.to_value().as_sql(false))
                .build();
            transaction.query(query).map_err(GreaseError::DbError)?;
        }

        let id_query = query_builder::select(Self::table_name())
            .fields(&["id"])
            .order_by("id", Order::Desc)
            .limit(num_events)
            .build();
        let new_ids = crate::db::load(&id_query, &mut transaction)?;

        if let Some(new_id) = new_ids.into_iter().nth(num_events) {
            transaction.commit().map_err(GreaseError::DbError)?;
            Ok(new_id)
        } else {
            Err(GreaseError::ServerError(
                "error inserting new event into database".to_owned(),
            ))
        }
    }

    pub fn update(event_id: i32, event_update: EventUpdate, conn: &mut Conn) -> GreaseResult<()> {
        fn to_value<'a, T: ToValue>(t: T) -> String {
            t.to_value().as_sql(false)
        }

        let mut transaction = conn.start_transaction(false, None, None).map_err(GreaseError::DbError)?;
        let event = Event::load(event_id, &mut transaction)?;
        if let Some(_gig) = event.gig {
            let update_gig_query = query_builder::update(Gig::table_name())
                .filter(&format!("event = {}", event_id))
                .set("performance_time", &to_value(event_update.performance_time.ok_or(GreaseError::BadRequest(
                    "performance time is required on events that are gigs".to_owned()))?
                ))
                .set("uniform", &to_value(event_update.uniform.ok_or(GreaseError::BadRequest(
                    "uniform is required on events that are gigs".to_owned()))?
                ))
                .set("contact_name", &to_value(event_update.contact_name))
                .set("contact_email", &to_value(event_update.contact_email))
                .set("contact_phone", &to_value(event_update.contact_phone))
                .set("price", &to_value(event_update.price))
                .set("uniform", &to_value(event_update.public.ok_or(GreaseError::BadRequest(
                    "uniform is required on events that are gigs".to_owned()))?
                ))
                .set("summary", &to_value(event_update.summary))
                .set("description", &to_value(event_update.description))
                .build();
            transaction.query(update_gig_query).map_err(GreaseError::DbError)?;
        } else if event_update.performance_time.is_some() || event_update.uniform.is_some() || event_update.contact_name.is_some()
            || event_update.contact_email.is_some() || event_update.contact_phone.is_some() || event_update.price.is_some()
            || event_update.public.is_some() || event_update.summary.is_some() || event_update.description.is_some() {
            let update_gig_query = query_builder::update(Gig::table_name())
                .filter(&format!("event = {}", event_id))
                .set("performance_time", &to_value(event_update.performance_time.ok_or(GreaseError::BadRequest(
                    "performance time is required on events that are gigs".to_owned()))?
                ))
                .set("uniform", &to_value(event_update.uniform.ok_or(GreaseError::BadRequest(
                    "uniform is required on events that are gigs".to_owned()))?
                ))
                .set("contact_name", &to_value(event_update.contact_name))
                .set("contact_email", &to_value(event_update.contact_email))
                .set("contact_phone", &to_value(event_update.contact_phone))
                .set("price", &to_value(event_update.price))
                .set("uniform", &to_value(event_update.public.ok_or(GreaseError::BadRequest(
                    "uniform is required on events that are gigs".to_owned()))?
                ))
                .set("summary", &to_value(event_update.summary))
                .set("description", &to_value(event_update.description))
                .build();
            transaction.query(update_gig_query).map_err(GreaseError::DbError)?;
        }

        let update_event_query = query_builder::update(Event::table_name())
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
            .set("section", &to_value(event_update.section))
            .build();
        transaction.query(update_event_query).map_err(GreaseError::DbError)?;

        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(event_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let delete_query = query_builder::delete(Self::table_name())
            .filter(&format!("id = {}", event_id))
            .build();
        conn.query(delete_query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl Gig {
    pub fn load(given_event_id: i32, conn: &mut Conn) -> GreaseResult<Option<Gig>> {
        Gig::first_opt(&format!("event = {}", given_event_id), conn)
    }
}

#[derive(Serialize, Deserialize)]
pub struct EventWithGig {
    pub event: Event,
    pub gig: Option<Gig>,
}

impl EventWithGig {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.event.id,
            "name": self.event.name,
            "semester": self.event.semester,
            "type": self.event.type_,
            "call_time": self.event.call_time,
            "release_time": self.event.release_time,
            "points": self.event.points,
            "comments": self.event.comments,
            "location": self.event.location,
            "gig_count": self.event.gig_count,
            "default_attend": self.event.default_attend,
            "section": self.event.section,
            "performance_time": self.gig.as_ref().map(|gig| gig.performance_time),
            "uniform": self.gig.as_ref().map(|gig| &gig.uniform),
            "contact_name": self.gig.as_ref().map(|gig| &gig.contact_name),
            "contact_email": self.gig.as_ref().map(|gig| &gig.contact_email),
            "contact_phone": self.gig.as_ref().map(|gig| &gig.contact_phone),
            "price": self.gig.as_ref().map(|gig| gig.price),
            "public": self.gig.as_ref().map(|gig| gig.public),
            "summary": self.gig.as_ref().map(|gig| &gig.summary),
            "description": self.gig.as_ref().map(|gig| &gig.description),
        })
    }

    pub fn to_json_full(&self, member: &Member, conn: &mut Conn) -> GreaseResult<Value> {
        let mut json_val = self.to_json();

        let uniform = if let Some(uniform) = self.gig.as_ref().map(|gig| &gig.uniform) {
            Some(Uniform::load(uniform, conn)?)
        } else {
            None
        };
        json_val["uniform"] = json!(uniform);

        let attendance = Attendance::load_for_member_at_event(&member.email, self.event.id, conn)?;
        json_val["attendance"] = json!(attendance);

        Ok(json_val)
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
pub struct EventWithGigRow {
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
    pub uniform: Option<String>,
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
