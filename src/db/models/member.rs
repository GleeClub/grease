use db::models::{ActiveSemester, Attendance, Event, Member, Semester, Session};
use db::schema::member::{
    self,
    dsl::{email, first_name, last_name, pass_hash},
};
use db::schema::{active_semester, member_role, role_permission, session};
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::{GreaseError, GreaseResult};
use serde_json::{json, Value};
use serde::Serialize;
use chrono::Local;

pub struct MemberForSemester {
    pub member: Member,
    pub active_semester: ActiveSemester,
}

impl Member {
    pub fn load(given_email: &str, conn: &MysqlConnection) -> GreaseResult<Member> {
        member::table
            .filter(email.eq(given_email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "no member exists with the email {}",
                given_email
            )))
    }

    pub fn check_login(given_email: &str, given_pass_hash: &str, conn: &MysqlConnection) -> GreaseResult<Option<Member>> {
        member::table
            .filter(email.eq(given_email).and(pass_hash.eq(given_pass_hash)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Member>> {
        member::table
            .order(first_name)
            .order(last_name)
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.preferred_name.as_ref().unwrap_or(&self.first_name),
            self.last_name
        )
    }

    pub fn as_changeset<'a>(&'a self) -> NewMember<'a> {
        NewMember {
            email: &self.email,
            first_name: &self.first_name,
            preferred_name: self.preferred_name.as_ref().map(|pn| pn.as_str()),
            last_name: &self.last_name,
            pass_hash: &self.pass_hash,
            phone_number: &self.phone_number,
            picture: self.picture.as_ref().map(|p| p.as_str()),
            passengers: self.passengers,
            location: &self.location,
            about: self.about.as_ref().map(|a| a.as_str()),
            major: self.major.as_ref().map(|m| m.as_str()),
            minor: self.minor.as_ref().map(|m| m.as_str()),
            hometown: self.hometown.as_ref().map(|h| h.as_str()),
            arrived_at_tech: self.arrived_at_tech,
            gateway_drug: self.gateway_drug.as_ref().map(|gd| gd.as_str()),
            conflicts: self.conflicts.as_ref().map(|c| c.as_str()),
            dietary_restrictions: self.dietary_restrictions.as_ref().map(|dr| dr.as_str()),
        }
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

    pub fn to_json_full(&self, conn: &MysqlConnection) -> GreaseResult<Value> {
        let mut json_val = self.to_json();
        let semesters = ActiveSemester::load_all_for_member(&self.email, conn)?
            .into_iter()
            .map(|found_active_semester| {
                let grades = self.calc_grades(&found_active_semester, conn)?;
                Ok(json!({
                    "semester": found_active_semester.semester,
                    "enrollment": found_active_semester.enrollment,
                    "section": found_active_semester.section,
                    "grades": grades
                }))
            }).collect::<GreaseResult<Vec<Value>>>()?;
        json_val["semesters"] = json!(semesters);

        Ok(json_val)
    }

    pub fn calc_grades(&self, active_semester: &ActiveSemester, conn: &MysqlConnection) -> GreaseResult<Grades> {
        let mut grade: f32 = 100.0;
        let all_events = Event::load_all_for_current_semester_until_now(conn)?;
        let semester_is_finished = Semester::load(&active_semester.semester, conn)?.end_date < Local::now().naive_local();
        let mut grade_items = Vec::new();
        let mut volunteer_gigs_attended = 0;

        for event in all_events {
            let attendance =
                Attendance::load_for_member_at_event(&self.email, event.id, conn)?;
            let went_to_sectionals =
                event.went_to_event_type_during_week_of(self, "sectional", conn)?;
            let went_to_rehearsal =
                event.went_to_event_type_during_week_of(self, "rehearsal", conn)?;

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
                            (release_time - event.call_time).num_minutes() as f32
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
                                format!("{:.2} points deducted for lateness to required event", delta),
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
                                    "Event grants $points-point bonus, but grade is capped at 100%"
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
                event,
                reason,
                change: point_change,
            });
        }

        Ok(Grades {
            final_grade: grade,
            changes: grade_items,
            volunteer_gigs_attended,
            semester_is_finished,
        })
    }
}

impl MemberForSemester {
    pub fn load(
        given_email: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberForSemester> {
        let found_member = Member::load(given_email, conn)?;

        match ActiveSemester::load(given_email, given_semester, conn)? {
            Some(active_semester) => Ok(MemberForSemester {
                member: found_member,
                active_semester,
            }),
            None => Err(GreaseError::NotActiveYet(found_member)),
        }
    }

    pub fn load_all(
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<MemberForSemester>> {
        member::table
            .inner_join(active_semester::table)
            .filter(active_semester::dsl::semester.eq(given_semester))
            .order((first_name, last_name))
            .load::<(Member, ActiveSemester)>(conn)
            .map_err(GreaseError::DbError)
            .map(|member_semester_pairs| {
                member_semester_pairs
                    .into_iter()
                    .map(|(found_member, found_active_semester)| MemberForSemester {
                        member: found_member,
                        active_semester: found_active_semester,
                    })
                    .collect()
            })
    }

    pub fn load_for_current_semester(
        given_email: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberForSemester> {
        let current_semester = Semester::load_current(conn)?;
        MemberForSemester::load(given_email, &current_semester.name, conn)
    }

    // TODO: make this one query
    pub fn load_from_token(
        grease_token: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberForSemester> {
        if let Some(member_session) = session::dsl::session
            .filter(session::dsl::key.eq(grease_token))
            .first::<Session>(conn)
            .optional()
            .map_err(GreaseError::DbError)?
        {
            MemberForSemester::load_for_current_semester(&member_session.member, &conn)
        } else {
            Err(GreaseError::Unauthorized)
        }
    }

    pub fn create(new_member: MemberForSemester, conn: &MysqlConnection) -> GreaseResult<String> {
        if let Ok(existing_member) = Member::load(&new_member.member.email, conn) {
            Err(GreaseError::BadRequest(format!(
                "A member with the email {} already exists.",
                existing_member.email
            )))
        } else {
            diesel::insert_into(member::table)
                .values(&new_member.member.as_changeset())
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            diesel::insert_into(active_semester::table)
                .values(&new_member.active_semester)
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            Attendance::create_for_new_member(&new_member.member.email, conn)?;

            Ok(new_member.member.email)
        }
    }

    pub fn num_volunteer_gigs(&self, conn: &MysqlConnection) -> GreaseResult<usize> {
        Attendance::load_for_member_at_all_events_of_type(&self.member.email, "volunteer", conn)
            .map(|attendance_pairs| {
                attendance_pairs
                    .iter()
                    .filter(|(attendance, _event)| attendance.did_attend)
                    .count()
            })
    }

    pub fn has_permission(&self, permission: &MemberPermission, conn: &MysqlConnection) -> GreaseResult<bool> {
        self.permissions(conn).map(|permissions| {
            permissions
                .iter()
                .find(|found_permission| found_permission == &permission)
                .is_some()
        })
    }

    pub fn permissions(&self, conn: &MysqlConnection) -> GreaseResult<Vec<MemberPermission>> {
        let role_permissions = member_role::table
            .inner_join(
                role_permission::table.on(role_permission::dsl::role.eq(member_role::dsl::role)),
            )
            .filter(
                member_role::dsl::member
                    .eq(&self.member.email)
                    .and(member_role::dsl::semester.eq(&self.active_semester.semester)),
            )
            .select((role_permission::dsl::permission, role_permission::dsl::event_type))
            .load::<(String, Option<String>)>(conn)
            .map_err(GreaseError::DbError)?;

        Ok(role_permissions.into_iter().map(|(permission, event_type)| {
            MemberPermission {
                name: permission,
                event_type,
            }
        }).collect())
    }

    pub fn positions(&self, conn: &MysqlConnection) -> GreaseResult<Vec<String>> {
        let current_semester = Semester::load_current(conn)?;
        member_role::table
            .filter(
                member_role::dsl::member
                    .eq(&self.member.email)
                    .and(member_role::dsl::semester.eq(&current_semester.name)),
            )
            .select(member_role::dsl::role)
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

#[derive(PartialEq)]
pub struct MemberPermission {
    pub name: String,
    pub event_type: Option<String>,
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
    pub event: Event,
    pub reason: String,
    pub change: f32,
}

impl ActiveSemester {
    pub fn load(
        given_email: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<ActiveSemester>> {
        use db::schema::active_semester::dsl::*;

        active_semester
            .filter(member.eq(given_email).and(semester.eq(given_semester)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_member(given_email: &str, conn: &MysqlConnection) -> GreaseResult<Vec<ActiveSemester>> {
        use db::schema::active_semester::dsl::*;
        use db::schema::semester;

        active_semester
            .inner_join(semester::table)
            .filter(member.eq(given_email))
            .order(semester::dsl::start_date)
            .select((member, semester, enrollment, section))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(
        new_active_semester: &ActiveSemester,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::active_semester::dsl::*;

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
            diesel::insert_into(active_semester)
                .values(new_active_semester)
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            Ok(())
        }
    }
}

#[derive(AsChangeset, Insertable)]
#[table_name = "member"]
pub struct NewMember<'a> {
    pub email: &'a str,
    pub first_name: &'a str,
    pub preferred_name: Option<&'a str>,
    pub last_name: &'a str,
    pub pass_hash: &'a str,
    pub phone_number: &'a str,
    pub picture: Option<&'a str>,
    pub passengers: i32,
    pub location: &'a str,
    pub about: Option<&'a str>,
    pub major: Option<&'a str>,
    pub minor: Option<&'a str>,
    pub hometown: Option<&'a str>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<&'a str>,
    pub conflicts: Option<&'a str>,
    pub dietary_restrictions: Option<&'a str>,
}
