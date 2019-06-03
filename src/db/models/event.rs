use chrono::Datelike;
use chrono::{Duration, NaiveDateTime};
use db::models::*;
use db::schema::event::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::result::QueryResult;
use diesel::*;
use serde_json::{to_value, Value};

impl Event {
    // pub const VALID_TYPES: [&'static str; 6] = [
    //     "Rehearsal",
    //     "Sectional",
    //     "Tutti",
    //     "Volunteer",
    //     "Ombuds",
    //     "Other",
    // ];

    pub fn load(given_event_id: i32, conn: &MysqlConnection) -> Result<Event, String> {
        event
            .filter(id.eq(given_event_id))
            .first::<Event>(conn)
            .optional()
            .expect("error loading event")
            .ok_or(format!("event with id {} doesn't exist", given_event_id))
    }

    pub fn load_all(conn: &MysqlConnection) -> Vec<Event> {
        event
            .order(performance_time)
            .load::<Event>(conn)
            .expect("error loading events")
    }

    pub fn load_all_of_type(given_category: &EventCategory, conn: &MysqlConnection) -> Vec<Event> {
        event
            .filter(category.eq(given_category))
            .order(performance_time)
            .load::<Event>(conn)
            .expect("error loading events")
    }

    pub fn points(&self) -> f32 {
        match self.category {
            "tutti" => 35.0,
            "rehearsal" | "sectional" => 10.0,
            "volunteer" | "ombuds" | "other" | _ => 5.0,
        }
    }

    pub fn create(new_event: &NewEvent, conn: &MysqlConnection) -> i32 {
        insert_into(event)
            .values(new_event)
            .execute(conn)
            .expect("error adding event");

        let new_event_id = event
            .filter(title.eq(&new_event.title))
            .filter(start_time.eq(&new_event.start_time))
            .first::<Event>(conn)
            .expect("error loading event")
            .id;

        Attendance::create_for_new_event(new_event_id, conn);

        new_event_id
    }

    pub fn grade(&self, user: &User, current_grade: f32, conn: &MysqlConnection) -> (f32, String) {
        let attendance = Attendance::load_for_user_at_event(&user.email, self.id, conn);
        let mut value = self.points();

        // if they attended the event
        if attendance.did_attend {
            // if it's a Tutti gig, check if they attended this week's rehearsal
            if self.category == "tutti" {
                let mut rehearsals = Event::load_all_of_type(&"rehearsal", conn);
                rehearsals.sort_by_key(|e| e.start_time);
                let last_rehearsal = rehearsals
                    .iter()
                    .take_while(|e| e.start_time < self.start_time)
                    .last();
                if let Some(last) = last_rehearsal {
                    let rehearsal_attendance =
                        Attendance::load_for_user_at_event(&user.email, last.id, conn);
                    if !rehearsal_attendance.did_attend && !rehearsal_attendance.is_excused(conn) {
                        return (
                            current_grade - value,
                            "Lost full points for unexcused absence \
                             from this week's rehearsal"
                                .to_owned(),
                        );
                    }
                }
            // If they already attended a previous sectional in the same week, award no points
            } else if self.category == "sectional" {
                let sectionals = self.load_sectionals_the_week_of(conn);
                let num_attended = sectionals
                    .iter()
                    .filter(|s| {
                        s.start_time < self.start_time
                            && Attendance::load_for_user_at_event(&user.email, s.id, conn)
                                .did_attend
                    })
                    .count();
                if num_attended > 0 {
                    return if current_grade + value > 100.0 {
                        (
                            100.0,
                            format!(
                                "Earned to over 100 points (the grade cap), so only earned \
                                 {:.2} points for attending a previous sectional the same week",
                                100.0 - current_grade
                            ),
                        )
                    } else {
                        (
                            current_grade + value,
                            format!(
                                "Earned {:.0} points for attending a previous \
                                 sectional the same week",
                                value
                            ),
                        )
                    };
                } else {
                    value = 0.0;
                }
            }

            let tardiness = attendance.minutes_late as f32 * 10.0 / 60.0;
            let new_grade = current_grade + value - tardiness;

            if attendance.minutes_late > 0 {
                if new_grade > 100.0 {
                    (
                        100.0,
                        format!(
                            "Earned to over 100 points (the grade cap), \
                             so only earned {:.2} points (lost {:.2} points for tardiness)",
                            100.0 - current_grade,
                            tardiness
                        ),
                    )
                } else {
                    (
                        new_grade,
                        format!(
                            "Earned {:.2} points (lost {:.2} points for tardiness)",
                            value - tardiness,
                            tardiness
                        ),
                    )
                }
            } else {
                if new_grade > 100.0 {
                    (
                        100.0,
                        format!(
                            "Earned to over 100 points (the grade cap), \
                             so only earned {:.2} points",
                            100.0 - current_grade
                        ),
                    )
                } else {
                    (
                        new_grade,
                        format!("Earned {:.0} points (full value)", value),
                    )
                }
            }
        // didn't attend but should have
        } else if attendance.should_attend {
            if self.category == "ombuds" {
                (
                    current_grade,
                    "Ombuds events aren't required, no points lost".to_owned(),
                )
            } else if self.category == "sectional" {
                let other_sectionals = self.load_sectionals_the_week_of(conn);
                if other_sectionals.iter().any(|s| {
                    let attendance = Attendance::load_for_user_at_event(&user.email, s.id, conn);
                    // TODO: do excuses matter?
                    attendance.did_attend || attendance.is_excused(conn)
                }) {
                    (
                        current_grade,
                        "No points lost as another sectional was \
                         attended or excused the same week"
                            .to_owned(),
                    )
                } else if other_sectionals
                    .iter()
                    .filter(|s| s.start_time > self.start_time)
                    .count() > 0
                {
                    (
                        current_grade - value,
                        format!(
                            "Lost {:.0} points for not attending a sectional the given week",
                            value
                        ),
                    )
                } else {
                    (
                        current_grade,
                        "Only the last sectional of the week deducts points".to_owned(),
                    )
                }
            } else if attendance.is_excused(conn) {
                (
                    current_grade,
                    "Didn't lose points as the absence was excused".to_owned(),
                )
            } else {
                (
                    current_grade - value,
                    format!(
                        "Lost {:.0} points (full value) for an unexcused absence",
                        value
                    ),
                )
            }
        // didn't attend and shouldn't have
        } else {
            (
                current_grade,
                "Lost no points for not attending when not expected".to_owned(),
            )
        }
    }

    // TODO: fix for weekend sectionals?
    pub fn load_sectionals_the_week_of(&self, conn: &MysqlConnection) -> Vec<Event> {
        let datetime = NaiveDateTime::from_timestamp(self.start_time as i64, 0);
        let sunday_diff = datetime.date().weekday().num_days_from_sunday();
        let last_sunday = (datetime - Duration::days(sunday_diff as i64)).timestamp() as i32;
        let next_sunday = last_sunday + 60 * 60 * 24 * 7; // add a week in seconds

        let mut sectionals = Event::load_all_of_type("sectional", conn);
        sectionals.retain(|e| last_sunday > e.start_time && e.start_time < next_sunday);
        sectionals
    }

    // pub fn override_table_with_values(
    //     new_vals: &Vec<NewEvent>,
    //     conn: &MysqlConnection,
    // ) -> QueryResult<()> {
    //     diesel::delete(event).execute(conn)?;
    //     diesel::sql_query("ALTER SEQUENCE events_id_seq RESTART").execute(conn)?;
    //     diesel::insert_into(event).values(new_vals).execute(conn)?;

    //     Ok(())
    // }
}
