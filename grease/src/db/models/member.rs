use auth::MemberPermission;
use bcrypt::{hash, verify};
use db::models::grades::Grades;
use db::schema::member::dsl::*;
use db::{
    ActiveSemester, ActiveSemesterUpdate, Attendance, Enrollment, Member, NewMember,
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

#[derive(Serialize)]
pub struct MemberForSemesterJsonFormat<'a> {
    #[serde(flatten)]
    pub member: &'a Member,
    pub enrollment: Option<&'a Enrollment>,
    pub section: Option<&'a String>,
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

    pub fn to_json_full_for_all_semesters(&self, conn: &MysqlConnection) -> GreaseResult<Value> {
        let mut json_val = json!(self);
        let semesters = ActiveSemester::load_all_for_member(&self.email, conn)?
            .iter()
            .map(|(active_semester, semester)| {
                let grades = Grades::for_member(&self, Some(&active_semester), &semester, conn)?;
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
            let (mut new_member, new_active_semester) = new_member.for_current_semester(conn)?;
            new_member.pass_hash = hash(&new_member.pass_hash, 10).map_err(|err| {
                GreaseError::BadRequest(format!(
                    "Unable to generate a hash from the given password: {}",
                    err
                ))
            })?;
            diesel::insert_into(member)
                .values(&new_member)
                .execute(conn)?;
            diesel::insert_into(active_semester::table)
                .values(&new_active_semester)
                .execute(conn)?;
            Attendance::create_for_new_member(&new_member.email, conn)?;

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
                    on_campus.eq(&form.on_campus),
                    conflicts.eq(&form.conflicts),
                    dietary_restrictions.eq(&form.dietary_restrictions),
                ))
                .execute(conn)?;

            let new_active_semester = ActiveSemester {
                member: given_email.clone(),
                semester: current_semester.name,
                enrollment: form.enrollment,
                section: Some(form.section),
            };
            diesel::insert_into(active_semester::table)
                .values(&new_active_semester)
                .execute(conn)?;
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
        .execute(conn)?;

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
                .select(email)
                .filter(email.eq(&update.email))
                .first::<String>(conn)
                .optional()?
                .is_some();
            if given_email != &update.email && member_with_same_email {
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
                    .execute(conn)?;

                if let Some(active_semester) = new_member.active_semester {
                    diesel::insert_into(active_semester::table)
                        .values(&active_semester)
                        .execute(conn)?;
                    Attendance::create_for_new_member(&new_member.member.email, conn)?;
                }

                Ok(new_member.member.email)
            })
        }
    }

    pub fn section<'m>(&'m self) -> Option<&'m str> {
        self.active_semester.as_ref().and_then(|active_semester| {
            active_semester
                .section
                .as_ref()
                .map(|section| section.as_str())
        })
    }

    pub fn to_json(&self) -> Value {
        json!(MemberForSemesterJsonFormat {
            member: &self.member,
            enrollment: self.active_semester.as_ref().map(|a_s| &a_s.enrollment),
            section: self
                .active_semester
                .as_ref()
                .and_then(|a_s| a_s.section.as_ref()),
        })
    }

    pub fn to_json_full(&self, conn: &MysqlConnection) -> GreaseResult<Value> {
        #[derive(Serialize)]
        struct JsonFormat<'m> {
            #[serde(flatten)]
            member: MemberForSemesterJsonFormat<'m>,
            permissions: Vec<MemberPermission>,
            positions: Vec<String>,
        }

        Ok(json!(JsonFormat {
            member: MemberForSemesterJsonFormat {
                member: &self.member,
                enrollment: self.active_semester.as_ref().map(|a_s| &a_s.enrollment),
                section: self
                    .active_semester
                    .as_ref()
                    .and_then(|a_s| a_s.section.as_ref()),
            },
            positions: self.member.positions(conn)?,
            permissions: self.member.permissions(conn)?,
        }))
    }

    pub fn to_json_with_grades(
        &self,
        semester: &Semester,
        conn: &MysqlConnection,
    ) -> GreaseResult<Value> {
        #[derive(Serialize)]
        struct JsonFormat<'m> {
            #[serde(flatten)]
            member: MemberForSemesterJsonFormat<'m>,
            grades: Grades,
            permissions: Vec<MemberPermission>,
            positions: Vec<String>,
        }
        Ok(json!(JsonFormat {
            member: MemberForSemesterJsonFormat {
                member: &self.member,
                enrollment: self.active_semester.as_ref().map(|a_s| &a_s.enrollment),
                section: self
                    .active_semester
                    .as_ref()
                    .and_then(|a_s| a_s.section.as_ref()),
            },
            grades: Grades::for_member(
                &self.member,
                self.active_semester.as_ref(),
                semester,
                conn
            )?,
            positions: self.member.positions(conn)?,
            permissions: self.member.permissions(conn)?,
        }))
    }
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
    ) -> GreaseResult<Vec<(ActiveSemester, Semester)>> {
        use db::schema::{active_semester, semester};

        let rows = active_semester::table
            .inner_join(semester::table)
            .filter(active_semester::member.eq(given_email))
            .order_by(semester::start_date.desc())
            .load::<(ActiveSemester, Semester)>(conn)?;

        Ok(rows)
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
                .execute(conn)?;

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

        conn.transaction(|| {
            if ActiveSemester::load(given_member, given_semester, conn)?.is_some() {
                if updated_semester.enrollment.is_some() {
                    diesel::update(active_semester_filter)
                        .set(&updated_semester)
                        .execute(conn)?;
                } else {
                    diesel::delete(active_semester_filter).execute(conn)?;
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
                    .execute(conn)?;
                Attendance::create_for_new_member(given_member, conn)?;
            }

            Ok(())
        })
    }
}

impl NewMember {
    pub fn for_current_semester(
        self,
        conn: &MysqlConnection,
    ) -> GreaseResult<(Member, ActiveSemester)> {
        Ok((
            Member {
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
            ActiveSemester {
                member: self.email,
                semester: Semester::load_current(conn)?.name,
                section: self.section,
                enrollment: self.enrollment.ok_or(GreaseError::BadRequest(
                    "New members cannot enroll as inactive.".to_owned(),
                ))?,
            },
        ))
    }
}
