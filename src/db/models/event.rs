use chrono::Datelike;
use chrono::{Duration, Local};
use db::models::*;
use db::schema::event::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventWithGig {
    pub event: Event,
    pub gig: Option<Gig>,
}

impl Event {
    // pub const VALID_TYPES: [&'static str; 6] = [
    //     "Rehearsal",
    //     "Sectional",
    //     "Tutti",
    //     "Volunteer",
    //     "Ombuds",
    //     "Other",
    // ];

    pub fn load(given_event_id: i32, conn: &MysqlConnection) -> GreaseResult<EventWithGig> {
        use crate::db::schema::event;

        let found_event = event::table
            .filter(id.eq(given_event_id))
            .first::<Event>(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "event with id {} doesn't exist",
                given_event_id
            )))?;
        let found_gig = Gig::load(given_event_id, conn)?;

        Ok(EventWithGig {
            event: found_event,
            gig: found_gig,
        })
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<EventWithGig>> {
        use crate::db::schema::event;

        let found_events = event::table
            .left_join(gig::table)
            .order(gig::dsl::performance_time)
            .load::<(Event, Option<Gig>)>(conn)
            .map_err(GreaseError::DbError)?;

        Ok(found_events
            .into_iter()
            .map(|(found_event, found_gig)| EventWithGig {
                event: found_event,
                gig: found_gig,
            })
            .collect())
    }

    pub fn load_all_for_current_semester(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventWithGig>> {
        use crate::db::schema::event;

        let current_semester = Semester::load_current(conn)?;
        let found_events = event::table
            .left_join(gig::table)
            .filter(event::dsl::semester.eq(&current_semester.name))
            .order(event::dsl::call_time)
            .load::<(Event, Option<Gig>)>(conn)
            .map_err(GreaseError::DbError)?;

        Ok(found_events
            .into_iter()
            .map(|(found_event, found_gig)| EventWithGig {
                event: found_event,
                gig: found_gig,
            })
            .collect())
    }

    pub fn load_all_for_current_semester_until_now(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Event>> {
        let current_semester = Semester::load_current(conn)?;
        let now = Local::now().naive_local();

        event
            .filter(semester.eq(&current_semester.name))
            .filter(release_time.lt(now))
            .order(call_time)
            .load::<Event>(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_of_type_for_current_semester(
        given_event_type: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventWithGig>> {
        use crate::db::schema::event;

        let current_semester = Semester::load_current(conn)?;
        let found_events = event::table
            .left_join(gig::table)
            .filter(
                type_
                    .eq(given_event_type)
                    .and(semester.eq(&current_semester.name)),
            )
            .order(event::dsl::call_time)
            .load::<(Event, Option<Gig>)>(conn)
            .map_err(GreaseError::DbError)?;

        Ok(found_events
            .into_iter()
            .map(|(found_event, found_gig)| EventWithGig {
                event: found_event,
                gig: found_gig,
            })
            .collect())
    }

    // pub fn create(new_event: NewEvent, conn: &MysqlConnection) -> GreaseResult<i32> {
    //     let new_event_id: i32 = insert_into(event)
    //         .values(new_event)
    //         .returning(id)
    //         .execute(conn)
    //         .map_err(GreaseError::DbError)? as i32;

    //     Attendance::create_for_new_event(new_event_id, conn)?;

    //     Ok(new_event_id)
    // }

    pub fn went_to_event_type_during_week_of(
        &self,
        member: &Member,
        given_event_type: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<bool>> {
        use db::schema::attendance;

        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);
        let now = Local::now().naive_local();

        let given_event_type_attendance_for_week = event
            .inner_join(attendance::table)
            .select((id, attendance::dsl::did_attend))
            .filter(
                type_
                    .eq(given_event_type)
                    .and(id.ne(self.id))
                    .and(semester.eq(&self.semester))
                    .and(call_time.gt(last_sunday))
                    .and(release_time.lt(next_sunday))
                    .and(release_time.lt(now))
                    .and(attendance::dsl::member.eq(&member.email)),
            )
            .order(call_time)
            .load::<(i32, bool)>(conn)
            .map_err(GreaseError::DbError)?;

        let attended_given_event_types = given_event_type_attendance_for_week
            .into_iter()
            .map(|(event_id, did_attend)| {
                if did_attend {
                    Ok(true)
                } else {
                    AbsenceRequest::excused_for_event(&member.email, event_id, conn)
                }
            })
            .collect::<GreaseResult<Vec<bool>>>()?;

        if attended_given_event_types.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(
                attended_given_event_types.into_iter().any(|sect| sect),
            ))
        }
    }

    // TODO: fix for weekend sectionals?
    pub fn load_sectionals_the_week_of(&self, conn: &MysqlConnection) -> GreaseResult<Vec<Event>> {
        let days_since_sunday = self.call_time.date().weekday().num_days_from_sunday() as i64;
        let last_sunday = self.call_time - Duration::days(days_since_sunday);
        let next_sunday = last_sunday + Duration::days(7);

        event
            .filter(
                type_
                    .eq("sectional")
                    .and(semester.eq(&self.semester))
                    .and(call_time.gt(last_sunday))
                    .and(release_time.lt(next_sunday)),
            )
            .order(call_time)
            .load::<Event>(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Gig {
    pub fn load(given_event_id: i32, conn: &MysqlConnection) -> GreaseResult<Option<Gig>> {
        use crate::db::schema::gig::dsl::*;

        gig.filter(event.eq(given_event_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }
}
