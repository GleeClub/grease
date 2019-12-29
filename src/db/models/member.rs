use auth::MemberPermission;
use bcrypt::{hash, verify};
use chrono::Local;
use db::schema::member::dsl::*;
use db::{
    AbsenceRequest, ActiveSemester, ActiveSemesterUpdate, Attendance, Event, Member, NewMember,
    RegisterForSemesterForm, Semester, Session,
};
use diesel::prelude::*;
use error::*;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Queryable, Debug, PartialEq)]
pub struct MemberForSemester {
    pub member: Member,
    pub active_semester: Option<ActiveSemester>,
}

impl Member {
    pub fn load(given_email: &str, conn: &MysqlConnection) -> GreaseResult<Member> {
        member
            .filter(email.eq(given_email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "No member with the email {}.",
                given_email
            )))
    }

    pub fn check_login(
        given_email: &str,
        given_pass_hash: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<Member>> {
        member
            .filter(email.eq(given_email))
            .first::<Member>(conn)
            .optional()
            .map_err(GreaseError::DbError)
            .map(|maybe_member| {
                maybe_member.filter(|given_member| {
                    verify(given_pass_hash, &given_member.pass_hash).unwrap_or(false)
                })
            })
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Member>> {
        member
            .order_by((last_name.asc(), first_name.asc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    /// Formats the member's full name.
    ///
    /// If the member's `preferred_name` is not `None`, then their full name is
    /// `<preferred_name> <last_name>`. Otherwise, it defaults to `<first_name> <last_name>`.
    // pub fn full_name(&self) -> String {
    //     format!(
    //         "{} {}",
    //         self.preferred_name
    //             .as_ref()
    //             .filter(|name| name.len() > 0)
    //             .unwrap_or(&self.first_name),
    //         self.last_name
    //     )
    // }

    /// Render this member's data to JSON, with some extra details.
    ///
    /// The extra field `semesters` is added, which is formatted as a
    /// list of objects in the below format:
    ///
    /// ```json
    /// {
    ///     "semester": string,
    ///     "enrollment": string,
    ///     "section": string,
    ///     "grades"L
    /// }
    /// ```
    ///
    /// if `active_semester` is not None, then
    pub fn to_json_full_for_semester(
        &self,
        active_semester: ActiveSemester,
        conn: &MysqlConnection,
    ) -> GreaseResult<Value> {
        let mut json_val = json!(self);
        json_val["semesters"] = json!([{
            "semester": &active_semester.semester,
            "enrollment": &active_semester.enrollment,
            "section": &active_semester.section,
            "grades": self.calc_grades(&active_semester, conn)?
        }]);
        json_val["permissions"] = json!(self.permissions(conn)?);
        json_val["positions"] = json!(self.positions(conn)?);

        Ok(json_val)
    }

    pub fn to_json_full_for_all_semesters(&self, conn: &MysqlConnection) -> GreaseResult<Value> {
        let mut json_val = json!(self);
        let semesters = ActiveSemester::load_all_for_member(&self.email, conn)?
            .iter()
            .map(|active_semester| {
                let grades = self.calc_grades(&active_semester, conn)?;
                Ok(json!({
                    "semester": &active_semester.semester,
                    "enrollment": &active_semester.enrollment,
                    "section": &active_semester.section,
                    "grades": grades
                }))
            })
            .collect::<GreaseResult<Vec<Value>>>()?;
        json_val["semesters"] = json!(semesters);
        json_val["permissions"] = json!(self.permissions(conn)?);
        json_val["positions"] = json!(self.positions(conn)?);

        Ok(json_val)
    }

    pub fn to_json_with_grades(
        &self,
        active_semester: Option<ActiveSemester>,
        conn: &MysqlConnection,
    ) -> GreaseResult<Value> {
        let mut json_val = json!(self);

        if let Some(ref active_semester) = active_semester {
            json_val["grades"] = json!(self.calc_grades(&active_semester, conn)?);
        };
        json_val["section"] = json!(&active_semester.as_ref().map(|a_s| &a_s.section));
        json_val["enrollment"] = json!(&active_semester.as_ref().map(|a_s| &a_s.enrollment));
        json_val["permissions"] = json!(self.permissions(conn)?);
        json_val["positions"] = json!(self.positions(conn)?);

        Ok(json_val)
    }

    pub fn calc_grades(
        &self,
        given_active_semester: &ActiveSemester,
        conn: &MysqlConnection,
    ) -> GreaseResult<Grades> {
        use db::schema::{absence_request, event};

        let now = Local::now().naive_local();
        let mut grade: f32 = 100.0;
        let event_attendance_pairs = Attendance::load_for_member_at_all_events(
            &self.email,
            &given_active_semester.semester,
            conn,
        )?;
        let given_semester = Semester::load(&given_active_semester.semester, conn)?;
        let semester_is_finished = given_semester.end_date < now;
        let gig_requirement = given_semester.gig_requirement as usize;
        let mut grade_items = Vec::new();
        let mut volunteer_gigs_attended = 0;
        let semester_absence_requests = event::table
            .inner_join(absence_request::table)
            .filter(
                absence_request::dsl::member
                    .eq(&self.email)
                    .and(event::dsl::semester.eq(&given_active_semester.semester)),
            )
            .load::<(Event, AbsenceRequest)>(conn)
            .map(|rows| rows.into_iter().map(|(_e, a)| a).collect())
            .map_err(GreaseError::DbError)?;

        let event_attendance_checks = event_attendance_pairs
            .iter()
            .filter_map(|(event, _attendance)| {
                if event.call_time < now {
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

                    Some((went_to_sectionals, went_to_rehearsal))
                } else {
                    None
                }
            })
            .collect::<Vec<(Option<bool>, Option<bool>)>>();

        for ((event, attendance), (went_to_sectionals, went_to_rehearsal)) in event_attendance_pairs
            .into_iter()
            .zip(event_attendance_checks.into_iter())
            .take_while(|((event, _attendance), _checks)| event.call_time < now)
        {
            let (point_change, reason) = {
                if attendance.did_attend {
                    let bonus_event = event.type_ == "Volunteer Gig"
                        || event.type_ == "Ombuds"
                        || (event.type_ == "Other" && !attendance.should_attend)
                        || (event.type_ == "Sectional" && went_to_sectionals.unwrap_or(false));
                    if !went_to_rehearsal.unwrap_or(event.type_ != "Rehearsal")
                        && ["Volunteer Gig", "Tutti Gig"].contains(&event.type_.as_str())
                    {
                        // If you haven't been to rehearsal this week, you can't get points or gig credit
                        if event.type_ == "Volunteer Gig" {
                            (0.0, format!("{}-point bonus denied because this week's rehearsal was missed", event.points))
                        } else {
                            (
                                -(event.points as f32),
                                "Full deduction for unexcused absence from this week's rehearsal"
                                    .to_owned(),
                            )
                        }
                    } else if attendance.minutes_late > 0 && event.type_ != "Ombuds" {
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
                            if event.type_ == "Volunteer Gig" && event.gig_count {
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
                        if event.type_ == "Volunteer Gig" && event.gig_count {
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
                    if event.type_ == "Ombuds" {
                        (
                            0.0,
                            "You do not lose points for missing an ombuds event".to_owned(),
                        )
                    } else if event.type_ == "Sectional" && went_to_sectionals == Some(true) {
                        (
                            0.0,
                            "No deduction because you attended a different sectional this week"
                                .to_owned(),
                        )
                    } else if event.type_ == "Sectional"
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
                attendance,
                change: point_change,
                partial_score: grade,
                reason,
            });
        }

        Ok(Grades {
            final_grade: grade,
            volunteer_gigs_attended,
            gig_requirement,
            semester_is_finished,
            changes: grade_items,
        })
    }

    pub fn create(new_member: NewMember, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::active_semester;

        let existing_member = member
            .filter(email.eq(&new_member.email))
            .first::<Member>(conn)
            .optional()?;

        if existing_member.is_some() {
            return Err(GreaseError::BadRequest(format!(
                "A member already exists with the email {}.",
                &new_member.email
            )));
        }
        conn.transaction(|| {
            let member_for_semester = new_member.for_current_semester(conn)?;
            diesel::insert_into(member)
                .values(&member_for_semester.member)
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            if let Some(new_active_semester) = member_for_semester.active_semester {
                diesel::insert_into(active_semester::table)
                    .values(&new_active_semester)
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;
                Attendance::create_for_new_member(&member_for_semester.member.email, conn)?;
            }

            Ok(())
        })
    }

    pub fn register_for_semester(
        given_email: String,
        form: RegisterForSemesterForm,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::active_semester;

        let current_semester = Semester::load_current(conn)?;
        if MemberForSemester::load(&given_email, &current_semester.name, conn)?
            .active_semester
            .is_some()
        {
            return Err(GreaseError::BadRequest(format!(
                "Member with email {} is already active for the current semester.",
                &given_email,
            )));
        }

        conn.transaction(|| {
            diesel::update(member.filter(email.eq(&given_email)))
                .set((
                    location.eq(&form.location),
                    conflicts.eq(&form.conflicts),
                    dietary_restrictions.eq(&form.dietary_restrictions),
                ))
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            // format!("No member with email {}.", &email),

            let new_active_semester = ActiveSemester {
                member: given_email.clone(),
                semester: current_semester.name,
                enrollment: form.enrollment,
                section: Some(form.section),
            };
            diesel::insert_into(active_semester::table)
                .values(&new_active_semester)
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            Attendance::create_for_new_member(&given_email, conn)?;

            Ok(())
        })
    }

    pub fn mark_inactive_for_semester(
        given_email: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::active_semester::dsl::{active_semester, member, semester};

        diesel::delete(
            active_semester.filter(member.eq(given_email).and(semester.eq(given_semester))),
        )
        .execute(conn)
        .map_err(GreaseError::DbError)?;

        Ok(())
        // format!("Member {} is not active for semester {}.", email, semester),
    }

    pub fn delete(given_email: &str, conn: &MysqlConnection) -> GreaseResult<()> {
        diesel::delete(member.filter(email.eq(given_email)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
        // format!("No member exists with email {}.", email),
    }

    pub fn update(
        given_email: &str,
        as_self: bool,
        update: NewMember,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        conn.transaction(|| {
            let member_with_same_email = member
                .filter(email.eq(given_email))
                .first::<Member>(conn)
                .optional()?;
            if given_email != &update.email && member_with_same_email.is_some() {
                return Err(GreaseError::BadRequest(format!(
                    "Cannot change email to {}, as another user has that email.",
                    &update.email
                )));
            }

            let given_member = Member::load(&given_email, conn)?;
            let member_pass_hash = if let Some(member_pass_hash) = update.pass_hash {
                if as_self {
                    hash(&member_pass_hash, 10).map_err(|err| {
                        GreaseError::BadRequest(format!(
                            "Unable to generate a hash from the given password: {}",
                            err
                        ))
                    })?
                } else {
                    return Err(GreaseError::BadRequest(
                        "Only members themselves can change their own passwords.".to_owned(),
                    ));
                }
            } else {
                given_member.pass_hash
            };

            diesel::update(member.filter(email.eq(given_email)))
                .set((
                    email.eq(&update.email),
                    first_name.eq(&update.first_name),
                    preferred_name.eq(&update.preferred_name),
                    last_name.eq(&update.last_name),
                    phone_number.eq(&update.phone_number),
                    picture.eq(&update.picture),
                    passengers.eq(&update.passengers),
                    location.eq(&update.location),
                    about.eq(&update.about),
                    major.eq(&update.major),
                    minor.eq(&update.minor),
                    hometown.eq(&update.hometown),
                    arrived_at_tech.eq(&update.arrived_at_tech),
                    gateway_drug.eq(&update.gateway_drug),
                    conflicts.eq(&update.conflicts),
                    dietary_restrictions.eq(&update.dietary_restrictions),
                    pass_hash.eq(member_pass_hash),
                ))
                .execute(conn)
                .map_err(GreaseError::DbError)?;
            // format!("No member with the email {} exists.", given_email),

            let current_semester = Semester::load_current(conn)?;
            let semester_update = ActiveSemesterUpdate {
                enrollment: update.enrollment,
                section: update.section,
            };
            ActiveSemester::update(&update.email, &current_semester.name, semester_update, conn)?;

            Ok(())
        })
    }

    pub fn permissions(&self, conn: &MysqlConnection) -> GreaseResult<Vec<MemberPermission>> {
        use db::schema::{member_role, role_permission};

        member_role::table
            .inner_join(role_permission::table.on(member_role::role.eq(role_permission::role)))
            .select((role_permission::permission, role_permission::event_type))
            .filter(member_role::member.eq(&self.email))
            .distinct()
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn positions(&self, conn: &MysqlConnection) -> GreaseResult<Vec<String>> {
        use db::schema::member_role;

        member_role::table
            .select(member_role::role)
            .filter(member_role::member.eq(&self.email))
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl MemberForSemester {
    pub fn load(
        given_email: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberForSemester> {
        Ok(MemberForSemester {
            member: Member::load(given_email, conn)?,
            active_semester: ActiveSemester::load(given_email, given_semester, conn)?,
        })
    }

    pub fn load_all(
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<MemberForSemester>> {
        use db::schema::active_semester;

        member
            .left_join(active_semester::table)
            .filter(active_semester::dsl::semester.eq(given_semester))
            .order_by((last_name.asc(), first_name.asc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_current_semester(
        given_email: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<MemberForSemester> {
        let current_semester = Semester::load_current(conn)?;
        MemberForSemester::load(given_email, &current_semester.name, conn)
    }

    pub fn load_from_token(token: &str, conn: &MysqlConnection) -> GreaseResult<MemberForSemester> {
        use db::schema::session::dsl::{key, session};

        let member_session = session
            .filter(key.eq(token))
            .first::<Session>(conn)
            .optional()
            .map_err(GreaseError::DbError)?;

        if let Some(member_session) = member_session {
            MemberForSemester::load_for_current_semester(&member_session.member, conn)
        } else {
            Err(GreaseError::Unauthorized)
        }
    }

    pub fn create(new_member: MemberForSemester, conn: &MysqlConnection) -> GreaseResult<String> {
        use db::schema::active_semester;

        if let Ok(existing_member) = Member::load(&new_member.member.email, conn) {
            Err(GreaseError::BadRequest(format!(
                "A member with the email {} already exists.",
                existing_member.email
            )))
        } else {
            conn.transaction(move || {
                diesel::insert_into(member)
                    .values(&new_member.member)
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;

                if let Some(active_semester) = new_member.active_semester {
                    diesel::insert_into(active_semester::table)
                        .values(&active_semester)
                        .execute(conn)
                        .map_err(GreaseError::DbError)?;
                    Attendance::create_for_new_member(&new_member.member.email, conn)?;
                }

                Ok(new_member.member.email)
            })
        }
    }

    pub fn to_json(&self) -> Value {
        let mut json = json!(self.member);
        json["enrollment"] = json!(self
            .active_semester
            .as_ref()
            .map(|active_semester| &active_semester.enrollment));
        json["section"] = json!(self
            .active_semester
            .as_ref()
            .and_then(|active_semester| active_semester.section.as_ref()));

        json
    }
}

#[derive(Serialize)]
pub struct Grades {
    #[serde(rename = "finalGrade")]
    pub final_grade: f32,
    pub changes: Vec<GradeChange>,
    #[serde(rename = "volunteerGigsAttended")]
    pub volunteer_gigs_attended: usize,
    #[serde(rename = "gigRequirement")]
    pub gig_requirement: usize,
    #[serde(rename = "semesterIsFinished")]
    pub semester_is_finished: bool,
}

#[derive(Serialize, Clone)]
pub struct GradeChange {
    pub event: Event,
    pub attendance: Attendance,
    pub reason: String,
    pub change: f32,
    #[serde(rename = "partialScore")]
    pub partial_score: f32,
}

impl ActiveSemester {
    pub fn load(
        given_email: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<ActiveSemester>> {
        use db::schema::active_semester::dsl::{
            active_semester, member as member_field, semester as semester_field,
        };

        active_semester
            .filter(
                member_field
                    .eq(given_email)
                    .and(semester_field.eq(given_semester)),
            )
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_member(
        given_email: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<ActiveSemester>> {
        use db::schema::active_semester::dsl::{active_semester, member as member_field};
        use db::schema::semester::dsl::{semester, start_date};

        active_semester
            .inner_join(semester)
            .filter(member_field.eq(given_email))
            .order_by(start_date.desc())
            .load::<(ActiveSemester, Semester)>(conn)
            .map(|rows| rows.into_iter().map(|(a, _)| a).collect())
            .map_err(GreaseError::DbError)
    }

    pub fn create(
        new_active_semester: &ActiveSemester,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::active_semester;

        let existing_active_semester = ActiveSemester::load(
            &new_active_semester.member,
            &new_active_semester.semester,
            conn,
        )?;

        if existing_active_semester.is_some() {
            Err(GreaseError::BadRequest(format!(
                "The member with email {} already is active in semester {}.",
                new_active_semester.member, new_active_semester.semester
            )))
        } else {
            diesel::insert_into(active_semester::table)
                .values(new_active_semester)
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            Ok(())
        }
    }

    pub fn update(
        given_member: &str,
        given_semester: &str,
        updated_semester: ActiveSemesterUpdate,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::active_semester::dsl::{
            active_semester, member as member_field, semester as semester_field,
        };

        let active_semester_filter = active_semester.filter(
            member_field
                .eq(given_member)
                .and(semester_field.eq(given_semester)),
        );

        if ActiveSemester::load(given_member, given_semester, conn)?.is_some() {
            if updated_semester.enrollment.is_some() {
                diesel::update(active_semester_filter)
                    .set(&updated_semester)
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;
            } else {
                diesel::delete(active_semester_filter)
                    .execute(conn)
                    .map_err(GreaseError::DbError)?;
            }
        } else if let Some(enrollment) = updated_semester.enrollment {
            let new_active_semester = ActiveSemester {
                member: given_member.to_owned(),
                semester: given_semester.to_owned(),
                section: updated_semester.section,
                enrollment,
            };
            diesel::insert_into(active_semester)
                .values(&new_active_semester)
                .execute(conn)
                .map_err(GreaseError::DbError)?;
        }

        Ok(())
    }
}

impl NewMember {
    pub fn for_current_semester(self, conn: &MysqlConnection) -> GreaseResult<MemberForSemester> {
        Ok(MemberForSemester {
            member: Member {
                email: self.email.clone(),
                first_name: self.first_name,
                preferred_name: self.preferred_name,
                last_name: self.last_name,
                pass_hash: self.pass_hash.ok_or(GreaseError::BadRequest(
                    "The `passHash` field is required for new member registration.".to_owned(),
                ))?,
                phone_number: self.phone_number,
                picture: self.picture,
                passengers: self.passengers,
                location: self.location,
                on_campus: self.on_campus,
                about: self.about,
                major: self.major,
                minor: self.minor,
                hometown: self.hometown,
                arrived_at_tech: self.arrived_at_tech,
                gateway_drug: self.gateway_drug,
                conflicts: self.conflicts,
                dietary_restrictions: self.dietary_restrictions,
            },
            active_semester: Some(ActiveSemester {
                member: self.email,
                semester: Semester::load_current(conn)?.name,
                section: self.section,
                enrollment: self.enrollment.ok_or(GreaseError::BadRequest(
                    "New members cannot enroll as inactive.".to_owned(),
                ))?,
            }),
        })
    }
}
