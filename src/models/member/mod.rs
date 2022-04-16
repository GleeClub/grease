use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::event::attendance::Attendance;
use crate::models::member::active_semester::{ActiveSemester, Enrollment, NewActiveSemester};
use crate::models::member::session::Session;
use crate::models::permissions::{MemberPermission, Role};
use crate::models::semester::Semester;

pub mod active_semester;
pub mod session;

#[derive(SimpleObject, Clone)]
pub struct Member {
    /// The member's email, which must be unique
    pub email: String,
    /// The member's first name
    pub first_name: String,
    /// The member's nick name
    pub preferred_name: Option<String>,
    /// The member's last name
    pub last_name: String,
    /// The member's phone number
    pub phone_number: String,
    /// An optional link to a profile picture for the member
    pub picture: Option<String>,
    /// How many people the member can drive to events (besides themself)
    pub passengers: i32,
    /// Where the member lives
    pub location: String,
    /// Whether the member lives on campus
    pub on_campus: Option<bool>,
    /// A short biography written by the member
    pub about: Option<String>,
    /// The member's academic major
    pub major: Option<String>,
    /// The member's academic minor
    pub minor: Option<String>,
    /// Where the member came from
    pub hometown: Option<String>,
    /// What year the member arrived at Georgia Tech
    pub arrived_at_tech: Option<i32>,
    /// What got them to join Glee Club
    pub gateway_drug: Option<String>,
    /// What conflicts with rehearsal the member may have
    pub conflicts: Option<String>,
    /// Any dietary restrictions the member may have
    pub dietary_restrictions: Option<String>,

    #[graphql(skip)]
    pub pass_hash: String,
}

#[ComplexObject]
impl Member {
    /// The member's full name
    pub async fn full_name(&self) -> String {
        format!(
            "{} {}",
            self.preferred_name.as_deref().unwrap_or(&self.first_name),
            self.last_name
        )
    }

    /// The semester TODO
    pub async fn semester(&self, ctx: &Context<'_>) -> Result<Option<ActiveSemester>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        ActiveSemester::for_member_during_semester(&self.email, &current_semester.name, conn).await
    }

    /// The officer positions currently held by the member
    pub async fn positions(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let conn = DbConn::from_ctx(ctx);
        Role::for_member(&self.email, conn).await
    }

    /// The permissions currently held by the member
    pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<MemberPermission>> {
        let conn = DbConn::from_ctx(ctx);
        MemberPermission::for_member(&self.email, conn).await
    }

    /// The semester TODO
    pub async fn semesters(&self, ctx: &Context<'_>) -> Result<Vec<ActiveSemester>> {
        let conn = DbConn::from_ctx(ctx);
        ActiveSemester::all_for_member(&self.email, conn).await
    }

    // /// The name of the semester they were active during
    // pub async fn semester(

    // @[GraphQL::Field(description: "The name of the semester they were active during")]
    // def semesters(context : UserContext) : Array(Models::ActiveSemester)
    //   context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

    //   (ActiveSemester.all_for_member @email).sort_by &.semester
    // end

    // @[GraphQL::Field(description: "The grades for the member in the given semester (default the current semester)")]
    // def grades(context : UserContext) : Models::Grades
    //   context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

    //   Grades.for_member self, Semester.current
    // end

    // @[GraphQL::Field(description: "All of the member's transactions for their entire time in Glee Club")]
    // def transactions(context : UserContext) : Array(Models::ClubTransaction)
    //   context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

    //   ClubTransaction.for_member_during_semester @email, Semester.current.name
    // end
}

impl Member {
    pub async fn with_email(email: &str, conn: &DbConn) -> Result<Member> {
        Self::with_email_opt(email, conn)
            .await?
            .ok_or_else(|| format!("No member with email {}", email).into())
    }

    pub async fn with_email_opt(email: &str, conn: &DbConn) -> Result<Option<Member>> {
        sqlx::query_as!(
            Member,
            "SELECT email, first_name, preferred_name, last_name, phone_number, picture, passengers,
                 location, on_campus as \"on_campus: bool\", about, major, minor, hometown,
                 arrived_at_tech, gateway_drug, conflicts, dietary_restrictions, pass_hash
             FROM member WHERE email = ?",
            email
        )
        .fetch_optional(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn with_token(token: &str, conn: &DbConn) -> Result<Member> {
        let session = Session::with_token(token, conn).await?;
        Self::with_email(&session.member, conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT email, first_name, preferred_name, last_name, phone_number, picture, passengers,
                 location, on_campus as \"on_campus: bool\", about, major, minor, hometown,
                 arrived_at_tech, gateway_drug, conflicts, dietary_restrictions, pass_hash
             FROM member ORDER BY last_name, first_name"
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    /// The members that were active during the given semester
    pub async fn active_during(semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT email, first_name, preferred_name, last_name, phone_number, picture, passengers,
                 location, on_campus as \"on_campus: bool\", about, major, minor, hometown,
                 arrived_at_tech, gateway_drug, conflicts, dietary_restrictions, pass_hash
             FROM member WHERE email IN
             (SELECT member FROM active_semester WHERE semester = ?)",
            semester
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn login_is_valid(email: &str, pass_hash: &str, conn: &DbConn) -> Result<bool> {
        if let Some(hash) =
            sqlx::query_scalar!("SELECT pass_hash FROM member WHERE email = ?", email)
                .fetch_optional(&mut *conn.get().await)
                .await?
        {
            bcrypt::verify(&hash, pass_hash)
                .map_err(|err| format!("Failed to verify password: {}", err).into())
        } else {
            Ok(false)
        }
    }

    pub async fn is_active(&self, email: &str, conn: &DbConn) -> Result<bool> {
        let current_semester = Semester::get_current(conn).await?;
        Ok(
            ActiveSemester::for_member_during_semester(email, &current_semester.name, conn)
                .await?
                .is_some(),
        )
    }

    pub async fn register(new_member: NewMember, conn: &DbConn) -> Result<()> {
        if sqlx::query!("SELECT email FROM member WHERE email = ?", new_member.email)
            .fetch_optional(&mut *conn.get().await)
            .await?
            .is_some()
        {
            return Err(
                format!("Another member already has the email {}", new_member.email).into(),
            );
        }

        let pass_hash = bcrypt::hash(new_member.pass_hash, 10)
            .map_err(|err| format!("Failed to hash password: {}", err))?;

        sqlx::query!(
            "INSERT INTO member
             (email, first_name, preferred_name, last_name, pass_hash, phone_number,
              picture, passengers, location, on_campus, about, major, minor, hometown,
              arrived_at_tech, gateway_drug, conflicts, dietary_restrictions)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            new_member.email,
            new_member.first_name,
            new_member.preferred_name,
            new_member.last_name,
            pass_hash,
            new_member.phone_number,
            new_member.picture,
            new_member.passengers,
            new_member.location,
            new_member.on_campus,
            new_member.about,
            new_member.major,
            new_member.minor,
            new_member.hometown,
            new_member.arrived_at_tech,
            new_member.gateway_drug,
            new_member.conflicts,
            new_member.dietary_restrictions
        )
        .execute(&mut *conn.get().await)
        .await?;

        let current_semester = Semester::get_current(conn).await?;
        Attendance::create_for_new_member(&new_member.email, &current_semester.name, conn).await
    }

    pub async fn register_for_current_semester(
        email: String,
        form: RegisterForSemesterForm,
        conn: &DbConn,
    ) -> Result<()> {
        let current_semester = Semester::get_current(conn).await?;
        ActiveSemester::create_for_member(
            form.active_semester(email.clone(), current_semester.name.clone()),
            conn,
        )
        .await?;

        sqlx::query!(
            "UPDATE member SET location = ?, on_campus = ?, conflicts = ?, dietary_restrictions = ?
             WHERE email = ?",
            form.location,
            form.on_campus,
            form.conflicts,
            form.dietary_restrictions,
            email
        )
        .execute(&mut *conn.get().await)
        .await?;

        Attendance::create_for_new_member(&email, &current_semester.name, conn).await
    }

    pub async fn update(
        email: &str,
        update: MemberUpdate,
        as_self: bool,
        conn: &DbConn,
    ) -> Result<()> {
        if email != &update.email {
            if sqlx::query!("SELECT email FROM member WHERE email = ?", update.email)
                .fetch_optional(&mut *conn.get().await)
                .await?
                .is_some()
            {
                return Err(format!(
                    "Cannot change email to {}, as another member has that email",
                    update.email
                )
                .into());
            }
        }

        let pass_hash = if let Some(new_hash) = update.pass_hash {
            // TODO: make as self enum
            if as_self {
                bcrypt::hash(new_hash, 10)?
            } else {
                return Err("Only members themselves can change their own passwords".into());
            }
        } else {
            sqlx::query_scalar!("SELECT pass_hash FROM member WHERE email = ?", email)
                .fetch_one(&mut *conn.get().await)
                .await?
        };

        sqlx::query!(
            "UPDATE member SET
             email = ?, first_name = ?, preferred_name = ?, last_name = ?,
             phone_number = ?, picture = ?, passengers = ?, location = ?,
             about = ?, major = ?, minor = ?, hometown = ?, arrived_at_tech = ?,
             gateway_drug = ?, conflicts = ?, dietary_restrictions = ?, pass_hash = ?",
            update.email,
            update.first_name,
            update.preferred_name,
            update.last_name,
            update.phone_number,
            update.picture,
            update.passengers,
            update.location,
            update.about,
            update.major,
            update.minor,
            update.hometown,
            update.arrived_at_tech,
            update.gateway_drug,
            update.conflicts,
            update.dietary_restrictions,
            pass_hash
        )
        .execute(&mut *conn.get().await)
        .await?;

        let current_semester = Semester::get_current(conn).await?;
        let active_semester_update = NewActiveSemester {
            member: email.to_owned(),
            semester: current_semester.name,
            enrollment: update.enrollment,
            section: update.section,
        };
        ActiveSemester::update(active_semester_update, conn).await
    }

    pub async fn delete(email: &str, conn: &DbConn) -> Result<()> {
        // TODO: verify exists
        Member::with_email(email, conn).await?;

        sqlx::query!("DELETE FROM member WHERE email = ?", email)
            .execute(&mut *conn.get().await)
            .await?;

        Ok(())
    }
}

#[derive(SimpleObject)]
pub struct SectionType {
    /// The name of the section (Tenor, Baritone, etc.)
    pub name: String,
}

impl SectionType {
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM section_type ORDER BY name")
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }
}

#[derive(InputObject)]
pub struct NewMember {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub on_campus: Option<bool>,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
    pub enrollment: Enrollment,
    pub section: Option<String>,
}

#[derive(InputObject)]
pub struct MemberUpdate {
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: Option<String>,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub on_campus: Option<bool>,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
    pub enrollment: Option<Enrollment>,
    pub section: Option<String>,
}

#[derive(InputObject)]
pub struct RegisterForSemesterForm {
    pub location: String,
    pub on_campus: Option<bool>,
    pub conflicts: String,
    pub dietary_restrictions: String,
    pub enrollment: Enrollment,
    pub section: String,
}

impl RegisterForSemesterForm {
    pub fn active_semester(&self, member: String, semester: String) -> NewActiveSemester {
        NewActiveSemester {
            member,
            semester,
            enrollment: Some(self.enrollment),
            section: Some(self.section.clone()),
        }
    }
}
