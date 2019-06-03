use db::models::*;
use db::schema::attendances::dsl::*;
use db::schema::users::dsl::{first_name, last_name};
use db::schema::EventCategory;
use db::schema::{attendances, events, users};
use diesel::pg::PgConnection;
use diesel::result::QueryResult;
use diesel::*;

impl Attendance {
    pub fn load(given_attendance_id: i32, conn: &PgConnection) -> Result<Attendance, String> {
        attendances
            .filter(id.eq(given_attendance_id))
            .first::<Attendance>(conn)
            .optional()
            .expect("error loading attendance")
            .ok_or(format!(
                "attendance with id {} not found",
                given_attendance_id
            ))
    }

    pub fn load_for_event(
        given_event_id: i32,
        conn: &PgConnection,
    ) -> Result<(Event, Vec<(Attendance, User)>), String> {
        let event = Event::load(given_event_id, conn)?;
        let attendance_data = attendances::table
            .inner_join(users::table)
            .order(first_name) // TODO: Which way is this supposed to go (first or last first)?
            .order(last_name)
            .filter(event_id.eq(&given_event_id))
            .load::<(Attendance, User)>(conn)
            .expect("error loading attendance");

        Ok((event, attendance_data))
    }

    pub fn load_for_event_separate_by_section(
        given_event_id: i32,
        conn: &PgConnection,
    ) -> Result<(Event, [Vec<(Attendance, User)>; 4]), String> {
        let (event, pairs) = Attendance::load_for_event(given_event_id, conn)?;
        let mut sorted = [Vec::new(), Vec::new(), Vec::new(), Vec::new()]; // TODO: figure out [T; n] notation here
        for pair in pairs {
            match pair.1.section.to_lowercase().as_str() {
                "tenor 1" => sorted[0].push(pair),
                "tenor 2" => sorted[1].push(pair),
                "baritone" => sorted[2].push(pair),
                "bass" => sorted[3].push(pair),
                bad => return Err(format!("{} is not a real section", bad)),
            }
        }

        Ok((event, sorted))
    }

    pub fn load_for_user_at_event(
        given_user_email: &str,
        given_event_id: i32,
        conn: &PgConnection,
    ) -> Attendance {
        attendances
            .filter(event_id.eq(&given_event_id))
            .filter(user_email.eq(&given_user_email))
            .first::<Attendance>(conn)
            .expect("error loading attendance")
    }

    pub fn load_for_user_at_all_events(
        given_user_email: &str,
        conn: &PgConnection,
    ) -> Vec<(Attendance, Event)> {
        attendances::table
            .inner_join(events::table)
            .filter(user_email.eq(&given_user_email))
            .load::<(Attendance, Event)>(conn)
            .expect("error loading event")
    }

    pub fn load_for_user_at_all_events_of_type(
        given_user_email: &str,
        event_type: &EventCategory,
        conn: &PgConnection,
    ) -> Vec<(Attendance, Event)> {
        attendances::table
            .inner_join(events::table)
            .filter(user_email.eq(&given_user_email))
            .filter(category.eq(event_type))
            .load::<(Attendance, Event)>(conn)
            .expect("error loading event")
    }

    pub fn create_for_new_user(given_user_email: &str, conn: &PgConnection) {
        let new_attendances = Event::load_all(conn)
            .iter()
            .map(|e| NewAttendance {
                user_email: given_user_email.to_owned(),
                event_id: e.id,
            })
            .collect::<Vec<NewAttendance>>();

        diesel::insert_into(attendances)
            .values(&new_attendances)
            .execute(conn)
            .expect("error adding new attendances");
    }

    pub fn create_for_new_event(given_event_id: i32, conn: &PgConnection) {
        let all_users = User::load_all(conn);
        let new_attendances = all_users
            .into_iter()
            .map(|u| NewAttendance {
                user_email: u.email,
                event_id: given_event_id,
            })
            .collect::<Vec<NewAttendance>>();

        diesel::insert_into(attendances)
            .values(&new_attendances)
            .execute(conn)
            .expect("error adding new attendances");
    }

    pub fn update(
        given_attendance_id: i32,
        attendance_form: &AttendanceForm,
        conn: &PgConnection,
    ) -> bool {
        let updated = diesel::update(attendances.find(given_attendance_id))
            .set(attendance_form)
            .get_result::<Attendance>(conn);

        updated.is_ok()
    }

    pub fn is_excused(&self, conn: &PgConnection) -> bool {
        AbsenceRequest::load(&self.user_email, self.event_id, conn)
            .and_then(|r| r.status)
            .unwrap_or(false)
    }

    pub fn override_table_with_values(
        new_vals: &Vec<NewAttendanceWithVals>,
        conn: &PgConnection,
    ) -> QueryResult<()> {
        diesel::delete(attendances).execute(conn)?;
        diesel::sql_query("ALTER SEQUENCE attendances_id_seq RESTART").execute(conn)?;
        diesel::insert_into(attendances)
            .values(new_vals)
            .execute(conn)?;

        Ok(())
    }
}
