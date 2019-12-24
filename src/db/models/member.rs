use auth::MemberPermission;
use bcrypt::{hash, verify};
use chrono::Local;
use db::*;
use error::*;
use serde::Serialize;
use serde_json::{json, Value};
use diesel::Connection;

#[derive(Debug, PartialEq)]
pub struct MemberForSemester {
    pub member: Member,
    pub active_semester: Option<ActiveSemester>,
}

impl Member {
    pub fn load<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<Member> {
        conn.first(
            &Member::filter(&format!("email = '{}'", email)),
            format!("No member with the email {}.", email),
        )
    }

    pub fn check_login<C: Connection>(
        email: &str,
        pass_hash: &str,
        conn: &mut C,
    ) -> GreaseResult<Option<Member>> {
        if let Some(member) =
            conn.first_opt::<Member>(&Member::filter(&format!("email = '{}'", email)))?
        {
            if verify(pass_hash, &member.pass_hash).unwrap_or(false) {
                Ok(Some(member))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Member>> {
        conn.load(&Member::select_all_in_order(
            "last_name, first_name",
            Order::Asc,
        ))
    }

    /// Formats the member's full name.
    ///
    /// If the member's `preferred_name` is not `None`, then their full name is
    /// `<preferred_name> <last_name>`. Otherwise, it defaults to `<first_name> <last_name>`.
    pub fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.preferred_name
                .as_ref()
                .filter(|name| name.len() > 0)
                .unwrap_or(&self.first_name),
            self.last_name
        )
    }

    /// Render this member's data to JSON.
    ///
    /// See the [JSON Format](crate::models::Member#json-format) above for what this returns.
    pub fn to_json(&self) -> Value {
        json!({
            "email": self.email,
            "firstName": self.first_name,
            "preferredName": self.preferred_name,
            "lastName": self.last_name,
            "fullName": self.full_name(),
            "phoneNumber": self.phone_number,
            "picture": self.picture,
            "passengers": self.passengers,
            "location": self.location,
            "onCampus": self.on_campus,
            "about": self.about,
            "major": self.major,
            "minor": self.minor,
            "hometown": self.hometown,
            "arrivedAtTech": self.arrived_at_tech,
            "gatewayDrug": self.gateway_drug,
            "conflicts": self.conflicts,
            "dietaryRestrictions": self.dietary_restrictions
        })
    }

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
    pub fn to_json_full_for_semester<C: Connection>(
        &self,
        active_semester: ActiveSemester,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();
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

    pub fn to_json_full_for_all_semesters<C: Connection>(
        &self,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();
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

    pub fn to_json_with_grades<C: Connection>(
        &self,
        active_semester: Option<ActiveSemester>,
        conn: &mut C,
    ) -> GreaseResult<Value> {
        let mut json_val = self.to_json();

        if let Some(ref active_semester) = active_semester {
            json_val["grades"] = json!(self.calc_grades(&active_semester, conn)?);
        };
        json_val["section"] = json!(&active_semester.as_ref().map(|a_s| &a_s.section));
        json_val["enrollment"] = json!(&active_semester.as_ref().map(|a_s| &a_s.enrollment));
        json_val["permissions"] = json!(self.permissions(conn)?);
        json_val["positions"] = json!(self.positions(conn)?);

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
        let semester = Semester::load(&active_semester.semester, conn)?;
        let semester_is_finished = semester.end_date < now;
        let gig_requirement = semester.gig_requirement as usize;
        let mut grade_items = Vec::new();
        let mut volunteer_gigs_attended = 0;
        let semester_absence_requests = conn.load(
            Select::new(Event::table_name())
                .join(AbsenceRequest::table_name(), "id", "event", Join::Inner)
                .fields(AbsenceRequest::field_names())
                .filter(&format!(
                    "member = '{}' AND semester = '{}'",
                    self.email, active_semester.semester
                )),
        )?;

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

    pub fn create(new_member: NewMember, conn: &mut DbConn) -> GreaseResult<()> {
        if conn
            .first_opt::<Member>(&Member::filter(&format!("email = '{}'", &new_member.email)))?
            .is_some()
        {
            Err(GreaseError::BadRequest(format!(
                "A member already exists with the email {}.",
                &new_member.email
            )))
        } else {
            conn.transaction(|transaction| {
                let MemberForSemester {
                    member,
                    active_semester,
                } = new_member.for_current_semester(transaction)?;
                member.insert(transaction)?;

                if let Some(active_semester) = active_semester {
                    active_semester.insert(transaction)?;
                    Attendance::create_for_new_member(&member.email, transaction)?;
                }

                Ok(())
            })
        }
    }

    pub fn register_for_semester(
        email: String,
        form: RegisterForSemesterForm,
        conn: &mut DbConn,
    ) -> GreaseResult<()> {
        let current_semester = Semester::load_current(conn)?;
        if MemberForSemester::load(&email, &current_semester.name, conn)?
            .active_semester
            .is_some()
        {
            return Err(GreaseError::BadRequest(format!(
                "Member with email {} is already active for the current semester.",
                &email,
            )));
        }

        conn.transaction(|transaction| {
            transaction.update(
                Update::new(Member::table_name())
                    .filter(&format!("email = '{}'", &email))
                    .set("location", &to_value(form.location))
                    .set("conflicts", &to_value(form.conflicts))
                    .set("dietary_restrictions", &to_value(form.dietary_restrictions)),
                format!("No member with email {}.", &email),
            )?;
            let new_active_semester = ActiveSemester {
                member: email.clone(),
                semester: current_semester.name,
                enrollment: form.enrollment,
                section: Some(form.section),
            };
            new_active_semester.insert(transaction)?;
            Attendance::create_for_new_member(&email, transaction)?;

            Ok(())
        })
    }

    pub fn mark_inactive_for_semester<C: Connection>(
        email: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Member::table_name())
                .filter(&format!("member = '{}'", email))
                .filter(&format!("semester = '{}'", semester)),
            format!("Member {} is not active for semester {}.", email, semester),
        )
    }

    pub fn delete<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Member::table_name()).filter(&format!("email = '{}'", email)),
            format!("No member exists with email {}.", email),
        )
    }

    pub fn update(
        email: &str,
        as_self: bool,
        update: NewMember,
        conn: &mut DbConn,
    ) -> GreaseResult<()> {
        conn.transaction(|transaction| {
            if email != &update.email
                && transaction
                    .first_opt::<Member>(&Member::filter(&format!("email = '{}'", &update.email)))?
                    .is_some()
            {
                return Err(GreaseError::BadRequest(format!(
                    "Cannot change email to {}, as another user has that email.",
                    &update.email
                )));
            }

            let member = Member::load(&email, transaction)?;
            let pass_hash = if let Some(pass_hash) = update.pass_hash {
                if as_self {
                    hash(&pass_hash, 10).map_err(|err| {
                        GreaseError::BadRequest(format!("Invalid password: {}", err))
                    })?
                } else {
                    return Err(GreaseError::BadRequest(
                        "Officers cannot change members' passwords.".to_owned(),
                    ));
                }
            } else {
                member.pass_hash
            };
            transaction.update(
                Update::new(Member::table_name())
                    .filter(&format!("email = '{}'", email))
                    .set("email", &to_value(&update.email))
                    .set("first_name", &to_value(&update.first_name))
                    .set("preferred_name", &to_value(&update.preferred_name))
                    .set("last_name", &to_value(&update.last_name))
                    .set("phone_number", &to_value(&update.phone_number))
                    .set("picture", &to_value(&update.picture))
                    .set("passengers", &to_value(&update.passengers))
                    .set("location", &to_value(&update.location))
                    .set("about", &to_value(&update.about))
                    .set("major", &to_value(&update.major))
                    .set("minor", &to_value(&update.minor))
                    .set("hometown", &to_value(&update.hometown))
                    .set("arrived_at_tech", &to_value(&update.arrived_at_tech))
                    .set("gateway_drug", &to_value(&update.gateway_drug))
                    .set("conflicts", &to_value(&update.conflicts))
                    .set(
                        "dietary_restrictions",
                        &to_value(&update.dietary_restrictions),
                    )
                    .set("pass_hash", &to_value(pass_hash)),
                format!("No member with the email {} exists.", email),
            )?;

            let current_semester = Semester::load_current(transaction)?;
            let semester_update = ActiveSemesterUpdate {
                enrollment: update.enrollment,
                section: update.section,
            };
            ActiveSemester::update(
                &update.email,
                &current_semester.name,
                semester_update,
                transaction,
            )?;

            Ok(())
        })
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
                .filter(&format!("member = '{}'", self.email)),
        )
    }

    pub fn positions<C: Connection>(&self, conn: &mut C) -> GreaseResult<Vec<String>> {
        conn.load(
            Select::new(MemberRole::table_name())
                .fields(&["role"])
                .filter(&format!("member = '{}'", self.email)),
        )
    }
}

impl MemberForSemester {
    pub fn load<C: Connection>(
        email: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
        let member = Member::load(email, conn)?;
        let active_semester = ActiveSemester::load(email, semester, conn)?;

        Ok(MemberForSemester {
            member,
            active_semester,
        })
    }

    pub fn load_all<C: Connection>(
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<MemberForSemester>> {
        conn.load_as::<MemberForSemesterRow, MemberForSemester>(
            Select::new(Member::table_name())
                .join(ActiveSemester::table_name(), "email", "member", Join::Left)
                .fields(MemberForSemesterRow::field_names())
                .filter(&format!("semester = '{}'", semester))
                .order_by("last_name, first_name", Order::Asc),
        )
    }

    pub fn load_for_current_semester<C: Connection>(
        given_email: &str,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
        let current_semester = Semester::load_current(conn)?;
        MemberForSemester::load(given_email, &current_semester.name, conn)
    }

    pub fn load_from_token<C: Connection>(
        token: &str,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
        if let Some(member_session) =
            conn.first_opt::<Session>(&Session::filter(&format!("`key` = '{}'", token)))?
        {
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
                if let Some(active_semester) = new_member.active_semester {
                    active_semester.insert(transaction)?;
                    Attendance::create_for_new_member(&new_member.member.email, transaction)?;
                }

                Ok(new_member.member.email)
            })
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "email": self.member.email,
            "firstName": self.member.first_name,
            "preferredName": self.member.preferred_name,
            "lastName": self.member.last_name,
            "fullName": self.member.full_name(),
            "phoneNumber": self.member.phone_number,
            "picture": self.member.picture,
            "passengers": self.member.passengers,
            "location": self.member.location,
            "onCampus": self.member.on_campus,
            "about": self.member.about,
            "major": self.member.major,
            "minor": self.member.minor,
            "hometown": self.member.hometown,
            "arrivedAtTech": self.member.arrived_at_tech,
            "gatewayDrug": self.member.gateway_drug,
            "conflicts": self.member.conflicts,
            "dietaryRestrictions": self.member.dietary_restrictions,
            "enrollment": self.active_semester.as_ref().map(|active_semester| &active_semester.enrollment),
            "section": self.active_semester.as_ref().and_then(|active_semester| active_semester.section.as_ref())
        })
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
    pub fn load<C: Connection>(
        email: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Option<ActiveSemester>> {
        conn.first_opt(&ActiveSemester::filter(&format!(
            "member = '{}' AND semester = '{}'",
            email, semester
        )))
    }

    pub fn load_all_for_member<C: Connection>(
        given_email: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<ActiveSemester>> {
        conn.load(
            Select::new(ActiveSemester::table_name())
                .join(
                    Semester::table_name(),
                    &format!("{}.semester", ActiveSemester::table_name()),
                    &format!("{}.name", Semester::table_name()),
                    Join::Inner,
                )
                .fields(ActiveSemester::field_names())
                .filter(&format!("member = '{}'", given_email))
                .order_by("start_date", Order::Desc),
        )
    }

    pub fn create<C: Connection>(
        new_active_semester: &ActiveSemester,
        conn: &mut C,
    ) -> GreaseResult<()> {
        if let Some(_existing) = ActiveSemester::load(
            &new_active_semester.member,
            &new_active_semester.semester,
            conn,
        )? {
            Err(GreaseError::BadRequest(format!(
                "The member with email {} already is active in semester {}.",
                new_active_semester.member, new_active_semester.semester
            )))
        } else {
            new_active_semester.insert(conn)
        }
    }

    pub fn update<C: Connection>(
        member: &str,
        semester: &str,
        updated_semester: ActiveSemesterUpdate,
        conn: &mut C,
    ) -> GreaseResult<()> {
        if ActiveSemester::load(member, semester, conn)?.is_some() {
            if let Some(enrollment) = updated_semester.enrollment {
                conn.update_opt(
                    Update::new(ActiveSemester::table_name())
                        .filter(&format!("member = '{}'", member))
                        .filter(&format!("semester = '{}'", semester))
                        .set("enrollment", &to_value(enrollment))
                        .set("section", &to_value(updated_semester.section)),
                )
            } else {
                conn.delete_opt(
                    Delete::new(ActiveSemester::table_name())
                        .filter(&format!("member = '{}'", member))
                        .filter(&format!("semester = '{}'", semester)),
                )
            }
        } else if let Some(enrollment) = updated_semester.enrollment {
            let new_active_semester = ActiveSemester {
                member: member.to_owned(),
                semester: semester.to_owned(),
                section: updated_semester.section,
                enrollment,
            };

            new_active_semester.insert(conn)
        } else {
            Ok(())
        }
    }
}

impl NewMember {
    pub fn for_current_semester<C: Connection>(
        self,
        conn: &mut C,
    ) -> GreaseResult<MemberForSemester> {
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
