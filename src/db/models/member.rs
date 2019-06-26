use chrono::Local;
use db::*;
use error::*;
use pinto::query_builder::*;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

pub struct MemberForSemester {
    pub member: Member,
    pub active_semester: ActiveSemester,
}

impl Member {
    pub fn load<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<Member> {
        conn.first(&Self::filter(&format!("email = '{}'", email)), format!("No member with the email {}.", email))
    }

    pub fn check_login<C: Connection>(
        email: &str,
        pass_hash: &str,
        conn: &mut C,
    ) -> GreaseResult<Option<Member>> {
        conn.first_opt(&Self::filter(&format!("email = '{}' AND pass_hash = '{}'", email, pass_hash)))
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Member>> {
        conn.load(&Self::select_all_in_order("last_name, first_name", Order::Asc))
    }

    pub fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.preferred_name.as_ref().unwrap_or(&self.first_name),
            self.last_name
        )
    }

    pub fn to_json(&self) -> Value {
        json!({
            "email": self.email,
            "first_name": self.first_name,
            "preferred_name": self.preferred_name,
            "last_name": self.last_name,
            "full_name": self.full_name(),
            "phone_number": self.phone_number,
            "picture": self.picture,
            "passengers": self.passengers,
            "location": self.location,
            "about": self.about,
            "major": self.major,
            "minor": self.minor,
            "hometown": self.hometown,
            "arrived_at_tech": self.arrived_at_tech,
            "gateway_drug": self.gateway_drug,
            "conflicts": self.conflicts,
            "dietary_restrictions": self.dietary_restrictions
        })
    }

    pub fn to_json_full<C: Connection>(
        &self,
        active_semester: Option<ActiveSemester>,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();
        let semesters = if let Some(active_semester) = active_semester {
            vec![active_semester]
        } else {
            ActiveSemester::load_all_for_member(&self.email, conn)?
        };
        let semesters = semesters
            .iter()
            .map(|found_active_semester| {
                let grades = self.calc_grades(&found_active_semester, conn)?;
                Ok(json!({
                    "semester": &found_active_semester.semester,
                    "enrollment": &found_active_semester.enrollment,
                    "section": &found_active_semester.section,
                    "grades": grades
                }))
            })
            .collect::<GreaseResult<Vec<Value>>>()?;
        json_val["semesters"] = json!(semesters);

        Ok(json_val)
    }

    pub fn to_json_with_grades<C: Connection>(
        &self,
        active_semester: Option<ActiveSemester>,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();
        let grades = if let Some(ref active_semester) = active_semester {
            Some(self.calc_grades(&active_semester, conn)?)
        } else {
            None
        };
        json_val["grades"] = json!(grades);
        json_val["section"] = json!(&active_semester.as_ref().map(|a_s| &a_s.section));
        json_val["enrollment"] = json!(&active_semester.as_ref().map(|a_s| &a_s.enrollment));

        Ok(json_val)
    }

    pub fn calc_grades<C: Connection>(
        &self,
        active_semester: &ActiveSemester,
        conn: &mut C,
    ) -> GreaseResult<Grades> {
        let now = Local::now().naive_local();
        let mut grade: f32 = 100.0;
        let event_attendance_pairs = Attendance::load_for_member_at_all_events(
            &self.email,
            &active_semester.semester,
            conn,
        )?;
        let semester_is_finished = Semester::load(&active_semester.semester, conn)?.end_date < now;
        let mut grade_items = Vec::new();
        let mut volunteer_gigs_attended = 0;
        let semester_absence_requests = conn.load(
            Select::new(Event::table_name())
                .join(AbsenceRequest::table_name(), "id", "event", Join::Inner)
                .fields(AbsenceRequest::field_names())
                .filter(&format!(
                    "member = '{}' AND semester = '{}'",
                    self.email, active_semester.semester
                ))
        )?;

        for (event, attendance) in event_attendance_pairs
            .iter()
            .take_while(|(event, _attendance)| event.call_time < now)
        {
            let went_to_sectionals = event.went_to_event_type_during_week_of(
                &event_attendance_pairs,
                &semester_absence_requests,
                "sectional",
            );
            let went_to_rehearsal = event.went_to_event_type_during_week_of(
                &event_attendance_pairs,
                &semester_absence_requests,
                "rehearsal",
            );

            let (point_change, reason) = {
                if attendance.did_attend {
                    let bonus_event = event.type_ == "volunteer"
                        || event.type_ == "ombuds"
                        || (event.type_ == "other" && !attendance.should_attend)
                        || (event.type_ == "sectional" && went_to_sectionals.unwrap_or(false));
                    if !went_to_rehearsal.unwrap_or(event.type_ != "rehearsal")
                        && ["volunteer", "tutti"].contains(&event.type_.as_str())
                    {
                        // If you haven't been to rehearsal this week, you can't get points or gig credit
                        if event.type_ == "volunteer" {
                            (0.0, format!("{}-point bonus denied because this week's rehearsal was missed", event.points))
                        } else {
                            (
                                -(event.points as f32),
                                "Full deduction for unexcused absence from this week's rehearsal"
                                    .to_owned(),
                            )
                        }
                    } else if attendance.minutes_late > 0 && event.type_ != "ombuds" {
                        // Lose points equal to the percentage of the event missed, if they should have attended
                        let event_duration = if let Some(release_time) = event.release_time {
                            if release_time <= event.call_time {
                                60.0
                            } else {
                                (release_time - event.call_time).num_minutes() as f32
                            }
                        } else {
                            60.0
                        };
                        let delta =
                            (attendance.minutes_late as f32 / event_duration) * event.points as f32;

                        if bonus_event {
                            if event.type_ == "volunteer" && event.gig_count {
                                volunteer_gigs_attended += 1;
                            }
                            if grade + event.points as f32 - delta > 100.0 {
                                (
                                    100.0 - grade,
                                    format!(
                                        "Event would grant {}-point bonus, \
                                         but {:.2} points deducted for lateness (capped at 100%)",
                                        event.points, delta
                                    ),
                                )
                            } else {
                                (
                                    event.points as f32 - delta,
                                    format!(
                                        "Event would grant {}-point bonus, \
                                         but {:.2} points deducted for lateness",
                                        event.points, delta
                                    ),
                                )
                            }
                        } else if attendance.should_attend {
                            (
                                -delta,
                                format!(
                                    "{:.2} points deducted for lateness to required event",
                                    delta
                                ),
                            )
                        } else {
                            (
                                0.0,
                                "No point change for attending required event".to_owned(),
                            )
                        }
                    } else if bonus_event {
                        if event.type_ == "volunteer" && event.gig_count {
                            volunteer_gigs_attended += 1;
                        }
                        // Get back points for volunteer gigs and and extra sectionals and ombuds events
                        if grade + event.points as f32 > 100.0 {
                            let point_change = 100.0 - grade;
                            (
                                point_change,
                                format!(
                                    "Event grants {:}-point bonus, but grade is capped at 100%",
                                    event.points
                                ),
                            )
                        } else {
                            (
                                event.points as f32,
                                "Full bonus awarded for attending volunteer or extra event"
                                    .to_owned(),
                            )
                        }
                    } else {
                        (
                            0.0,
                            "No point change for attending required event".to_owned(),
                        )
                    }
                } else if attendance.should_attend {
                    // Lose the full point value if did not attend
                    if event.type_ == "ombuds" {
                        (
                            0.0,
                            "You do not lose points for missing an ombuds event".to_owned(),
                        )
                    } else if event.type_ == "sectional" && went_to_sectionals == Some(true) {
                        (
                            0.0,
                            "No deduction because you attended a different sectional this week"
                                .to_owned(),
                        )
                    } else if event.type_ == "sectional"
                        && went_to_sectionals.is_none()
                        && event.load_sectionals_the_week_of(conn)?.len() > 1
                    {
                        (
                            0.0,
                            "No deduction because not all sectionals occurred yet".to_owned(),
                        )
                    } else {
                        (
                            -(event.points as f32),
                            "Full deduction for unexcused absence from event".to_owned(),
                        )
                    }
                } else {
                    (0.0, "Did not attend and not expected to".to_owned())
                }
            };

            grade += point_change;
            // Prevent the grade from ever rising above 100
            if grade > 100.0 {
                grade = 100.0;
            } else if grade < 0.0 {
                grade = 0.0;
            }

            grade_items.push(GradeChange {
                event: event.minimal(),
                did_attend: attendance.did_attend,
                should_attend: attendance.should_attend,
                change: point_change,
                reason,
            });
        }

        Ok(Grades {
            final_grade: grade,
            volunteer_gigs_attended,
            semester_is_finished,
            changes: grade_items,
        })
    }
}

impl MemberForSemester {
    pub fn load<C: Connection>(
        email: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
        let found_member = Member::load(email, conn)?;

        match ActiveSemester::load(email, semester, conn)? {
            Some(active_semester) => Ok(MemberForSemester {
                member: found_member,
                active_semester,
            }),
            None => Err(GreaseError::NotActiveYet(found_member)),
        }
    }

    pub fn load_all<C: Connection>(
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<MemberForSemester>> {
        conn.load_as::<MemberForSemesterRow, MemberForSemester>(
            Select::new(Member::table_name())
                .join(ActiveSemester::table_name(), "email", "member", Join::Inner)
                .fields(MemberForSemesterRow::field_names())
                .filter(&format!("semester = '{}'", semester))
                .order_by("last_name, first_name", Order::Asc)
        )
    }

    pub fn load_for_current_semester<C: Connection>(
        given_email: &str,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
        let current_semester = Semester::load_current(conn)?;
        MemberForSemester::load(given_email, &current_semester.name, conn)
    }

    pub fn load_from_token<C: Connection>(token: &str, conn: &mut C) -> GreaseResult<MemberForSemester> {
        if let Some(member_session) = conn.first_opt::<Session>(&Session::filter(&format!("`key` = '{}'", token)))? {
            MemberForSemester::load_for_current_semester(&member_session.member, conn)
        } else {
            Err(GreaseError::Unauthorized)
        }
    }

    pub fn create(new_member: MemberForSemester, conn: &mut DbConn) -> GreaseResult<String> {
        if let Ok(existing_member) = Member::load(&new_member.member.email, conn) {
            Err(GreaseError::BadRequest(format!(
                "A member with the email {} already exists.",
                existing_member.email
            )))
        } else {
            conn.transaction(move |transaction| {
                new_member.member.insert(transaction)?;
                new_member.active_semester.insert(transaction)?;
                Attendance::create_for_new_member(&new_member.member.email, transaction)?;

                Ok(new_member.member.email)
            })
        }
    }

    pub fn permissions<C: Connection>(&self, conn: &mut C) -> GreaseResult<Vec<MemberPermission>> {
        conn.load_as::<(String, Option<String>), MemberPermission>(
            Select::new(MemberRole::table_name())
                .join(
                    RolePermission::table_name(),
                    &format!("{}.role", MemberRole::table_name()),
                    &format!("{}.role", RolePermission::table_name()),
                    Join::Inner,
                )
                .fields(&["permission", "event_type"])
                .filter(&format!("member = '{}'", self.member.email))
        )
    }

    pub fn positions<C: Connection>(&self, conn: &mut C) -> GreaseResult<Vec<String>> {
        conn.load(
            Select::new(MemberRole::table_name())
                .fields(&["role"])
                .filter(&format!("member = '{}'", self.member.email))
        )
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, grease_derive::Extract)]
pub struct MemberPermission {
    pub name: String,
    pub event_type: Option<String>,
}

impl Into<MemberPermission> for (String, Option<String>) {
    fn into(self) -> MemberPermission {
        MemberPermission {
            name: self.0,
            event_type: self.1,
        }
    }
}

#[derive(Serialize)]
pub struct Grades {
    pub final_grade: f32,
    pub changes: Vec<GradeChange>,
    pub volunteer_gigs_attended: usize,
    pub semester_is_finished: bool,
}

#[derive(Serialize)]
pub struct GradeChange {
    pub event: Value,
    pub did_attend: bool,
    pub should_attend: bool,
    pub reason: String,
    pub change: f32,
}

impl ActiveSemester {
    pub fn load<C: Connection>(
        email: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Option<ActiveSemester>> {
        conn.first_opt(&Self::filter(&format!("member = '{}' AND semester = '{}'", email, semester)))
    }

    pub fn load_all_for_member<C: Connection>(
        given_email: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<ActiveSemester>> {
        conn.load(
            Select::new(Self::table_name())
                .join(
                    Semester::table_name(),
                    &format!("{}.semester", Self::table_name()),
                    &format!("{}.name", Semester::table_name()),
                    Join::Inner,
                )
                .fields(Self::field_names())
                .filter(&format!("member = '{}'", given_email))
                .order_by("start_date", Order::Desc)
        )
    }

    pub fn create<C: Connection>(new_active_semester: &ActiveSemester, conn: &mut C) -> GreaseResult<()> {
        if let Some(_existing) = Self::load(
            &new_active_semester.member,
            &new_active_semester.semester,
            conn,
        )? {
            Err(GreaseError::BadRequest(format!(
                "the member with email {} already is active in semester {}",
                new_active_semester.member, new_active_semester.semester
            )))
        } else {
            new_active_semester.insert(conn)
        }
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
pub struct MemberForSemesterRow {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
    pub member: String,
    pub semester: String,
    pub enrollment: Enrollment,
    pub section: Option<String>,
}

impl Into<MemberForSemester> for MemberForSemesterRow {
    fn into(self) -> MemberForSemester {
        MemberForSemester {
            member: Member {
                email: self.email,
                first_name: self.first_name,
                preferred_name: self.preferred_name,
                last_name: self.last_name,
                pass_hash: self.pass_hash,
                phone_number: self.phone_number,
                picture: self.picture,
                passengers: self.passengers,
                location: self.location,
                about: self.about,
                major: self.major,
                minor: self.minor,
                hometown: self.hometown,
                arrived_at_tech: self.arrived_at_tech,
                gateway_drug: self.gateway_drug,
                conflicts: self.conflicts,
                dietary_restrictions: self.dietary_restrictions,
            },
            active_semester: ActiveSemester {
                member: self.member,
                semester: self.semester,
                enrollment: self.enrollment,
                section: self.section,
            },
        }
    }
}