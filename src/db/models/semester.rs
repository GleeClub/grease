use chrono::naive::NaiveDate;
use chrono::{Duration, Local};
use db::models::*;
use db::schema::semester::dsl::*;
use diesel::pg::PgConnection;
use diesel::*;
use io::write_to_file;
use serde::Deserialize;
use serde_json;
use std::fs::read_to_string;

impl Semester {
    pub fn load(semester_id: i32, conn: &PgConnection) -> Result<Semester, String> {
        semester
            .filter(id.eq(semester_id))
            .first(conn)
            .optional()
            .expect("error loading semester")
            .ok_or(format!("No semester with id {}", semester_id))
    }

    pub fn generate_record_file(&self, conn: &PgConnection) -> String {
        LoadedSemester::for_semester(self, conn).into_string()
    }

    pub fn load_most_recent(conn: &PgConnection) -> Semester {
        semester
            .order_by(start_date.desc())
            .first(conn)
            .optional()
            .expect("error loading most recent semester")
            .unwrap_or_else(|| {
                let new_semester = NewSemester {
                    name: "New Semester".to_owned(),
                    filename: None,
                    start_date: Local::today().naive_local(),
                    end_date: (Local::today() + Duration::weeks(12)).naive_local(),
                };
                let new_id = Semester::create(new_semester, conn);
                Semester::load(new_id, conn).unwrap()
            })
    }

    pub fn create(new_semester: NewSemester, conn: &PgConnection) -> i32 {
        diesel::insert_into(semester)
            .values(new_semester)
            .returning(id)
            .get_result(conn)
            .expect("error creating new semester")
    }

    pub fn save_to_file(&mut self, conn: &PgConnection) -> Result<(), &'static str> {
        let file_str = self.generate_record_file(conn);
        let new_filename = format!("/static/semester_records/{}.json", self.name);
        write_to_file(file_str.into(), &new_filename)
            .map_err(|_e| "Error saving semester record to disk")?;
        diesel::update(semester.filter(id.eq(self.id)))
            .set(filename.eq(&new_filename))
            .execute(conn)
            .map_err(|_e| "Error updating semester record path in database")?;
        self.filename = Some(new_filename);

        Ok(())
    }

    pub fn change_semester(diff_semester_id: i32, conn: &PgConnection) -> Result<(), &'static str> {
        let mut current_semester =
            if let Some(current_semester_id) = Var::get::<i32>("CURRENT_SEMESTER_ID", conn) {
                if current_semester_id == diff_semester_id {
                    return Err("Cannot change to the current semester");
                } else {
                    Semester::load(current_semester_id, conn)
                        .map_err(|_e| "The current semester was set improperly")?
                }
            } else {
                Semester::load_most_recent(conn)
            };
        current_semester.save_to_file(conn)?;

        if let Ok(semester) = Semester::load(diff_semester_id, conn) {
            if let Some(file) = semester
                .filename
                .and_then(|fname| read_to_string(fname).ok())
            {
                if let Ok(loaded_semester) = LoadedSemester::try_load_from_str(&file) {
                    if conn
                        .transaction::<_, diesel::result::Error, _>(|| {
                            AbsenceRequest::override_table_with_values(
                                &loaded_semester.absence_requests,
                                conn,
                            )?;
                            Attendance::override_table_with_values(
                                &loaded_semester.attendances,
                                conn,
                            )?;
                            Carpool::override_table_with_values(&loaded_semester.carpools, conn)?;
                            Event::override_table_with_values(&loaded_semester.events, conn)?;
                            Ok(())
                        })
                        .is_ok()
                    {
                        Var::set("CURRENT_SEMESTER_ID", &diff_semester_id, conn);
                        Ok(())
                    } else {
                        Err("Error transitioning between semesters")
                    }
                } else {
                    Err("The new semester was saved incorrectly and can't be loaded")
                }
            } else {
                Err("No record file was found/usable for the new semester")
            }
        } else {
            Err("The new semester couldn't be found in our database")
        }
    }
}

struct LoadedSemester {
    name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    absence_requests: Vec<NewAbsenceRequest>,
    attendances: Vec<NewAttendanceWithVals>,
    carpools: Vec<NewCarpool>,
    events: Vec<NewEvent>,
}

impl LoadedSemester {
    fn try_load_from_str(file_str: &str) -> Result<LoadedSemester, &'static str> {
        let json = serde_json::from_str::<serde_json::Value>(file_str)
            .map_err(|_e| "Couldn't parse file as JSON")?;
        let json = json.as_object().ok_or("JSON not formatted correctly")?;

        Ok(LoadedSemester {
            name: json
                .get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.to_owned())
                .ok_or("Name of semester not specified")?,
            start_date: json
                .get("start_date")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse().ok())
                .ok_or("Start date not specified")?,
            end_date: json
                .get("end_date")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse().ok())
                .ok_or("End date not specified")?,
            absence_requests: json
                .get("absence_requests")
                .and_then(|ars| ars.as_array())
                .ok_or("Absence requests were not given as an array")?
                .iter()
                .map(|ar| {
                    NewAbsenceRequest::deserialize(ar)
                        .map_err(|_e| "One of the absence requests was malformed")
                })
                .collect::<Result<Vec<NewAbsenceRequest>, &'static str>>()?,
            attendances: json
                .get("attendances")
                .and_then(|a_s| a_s.as_array())
                .ok_or("Attendances were not given as an array")?
                .iter()
                .map(|a| {
                    NewAttendanceWithVals::deserialize(a)
                        .map_err(|_e| "One of the attendances was malformed")
                })
                .collect::<Result<Vec<NewAttendanceWithVals>, &'static str>>()?,
            carpools: json
                .get("carpools")
                .and_then(|cs| cs.as_array())
                .ok_or("Carpools were not given as an array")?
                .iter()
                .map(|c| {
                    NewCarpool::deserialize(c).map_err(|_e| "One of the carpools was malformed")
                })
                .collect::<Result<Vec<NewCarpool>, &'static str>>()?,
            events: json
                .get("events")
                .and_then(|es| es.as_array())
                .ok_or("Events were not given as an array")?
                .iter()
                .map(|e| NewEvent::deserialize(e).map_err(|_e| "One of the events was malformed"))
                .collect::<Result<Vec<NewEvent>, &'static str>>()?,
        })
    }

    // required tables:
    //   - absence_requests
    //   - attendances
    //   - carpools
    //   - events
    fn for_semester(semester: &Semester, conn: &PgConnection) -> LoadedSemester {
        use db::schema::{absence_requests, attendances, carpools, events};

        LoadedSemester {
            name: semester.name.clone(),
            start_date: semester.start_date.clone(),
            end_date: semester.end_date.clone(),
            absence_requests: absence_requests::table
                .load::<AbsenceRequest>(conn)
                .expect("error loading absence requests")
                .into_iter()
                .map(|ar| NewAbsenceRequest::from(ar))
                .collect(),
            attendances: attendances::table
                .load::<Attendance>(conn)
                .expect("error loading attendances")
                .into_iter()
                .map(|a| NewAttendanceWithVals::from(a))
                .collect(),
            carpools: carpools::table
                .load::<Carpool>(conn)
                .expect("error loading carpools")
                .into_iter()
                .map(|c| NewCarpool::from(c))
                .collect(),
            events: events::table
                .load::<Event>(conn)
                .expect("error loading events")
                .into_iter()
                .map(|e| NewEvent::from(e))
                .collect(),
        }
    }

    fn into_string(&self) -> String {
        serde_json::to_string(&json!({
            "name": self.name,
            "start_date": self.start_date,
            "end_date": self.end_date,
            "absence_requests": self.absence_requests,
            "attendances": self.attendances,
            "carpools": self.carpools,
            "events": self.events,
        })).unwrap()
    }
}
