use async_graphql::{ComplexObject, SimpleObject, Result};
use crate::db_conn::DbConn;

#[derive(SimpleObject)]
pub struct Member {
    /// The member's email, which must be unique
    pub email:                String,
    /// The member's first name
    pub first_name:           String,
    /// The member's nick name
    pub preferred_name:       Option<String>,
    /// The member's last name
    pub last_name:            String,
    /// The member's phone number
    pub phone_number:         String,
    /// An optional link to a profile picture for the member
    pub picture:              Option<String>,
    /// How many people the member can drive to events (besides themself)
    pub passengers:           i32,
    /// Where the member lives
    pub location:             String,
    /// Whether the member lives on campus
    pub on_campus:            Option<bool>,
    /// A short biography written by the member
    pub about:                Option<String>,
    /// The member's academic major
    pub major:                Option<String>,
    /// The member's academic minor
    pub minor:                Option<String>,
    /// Where the member came from
    pub hometown:             Option<String>,
    /// What year the member arrived at Georgia Tech
    pub arrived_at_tech:      Option<Int32>,
    /// What got them to join Glee Club
    pub gateway_drug:         Option<String>,
    /// What conflicts with rehearsal the member may have
    pub conflicts:            Option<String>,
    /// Any dietary restrictions the member may have
    pub dietary_restrictions: Option<String>,

    #[graphql(skip)]
    pub pass_hash:            String,
}

#[ComplexObject]
impl Member {
    /// The member's full name
    pub async fn full_name(&self) -> String {
        self.full_name_inner()
    }

    /// The semester TODO
    pub async fn semester(&self) -> Result<Option<String>> {
        self.semester()
    }

    /// The officer positions currently held by the member
    pub async fn positions(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Role::for_member(&self.email, conn).await
    }

    /// The permissions currently held by the member
    pub async fn positions(&self, ctx: &Context<'_>) -> Result<Vec<MemberPermission>> {
        let conn = ctx.data_unchecked::<DbConn>();
        MemberPermission::for_member(&self.email, conn).await
    }
}

    /// The name of the semester they were active during
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

// @semesters : Hash(String, ActiveSemester)?

impl Member {
    pub async fn with_email_opt(email: &str, conn: &DbConn) -> Result<Option<Member>> {
        sqlx::query_as!(Member, "SELECT * FROM member WHERE email = ?", email).query_optional(&mut *conn).await.into()
    }

    pub async fn with_email(email: &str, conn: &DbConn) -> Result<Member> {
        Self::with_email_opt(email, conn).and_then(|res| res.ok_or_else(|| format!("No member with email {}", email)))
    }

    pub async fn with_token(token: &str, conn: &DbConn) -> Result<Member> {
        let session = Session::for_token(token, conn).await?;
        Self::with_email(&session.member, conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM member ORDER BY last_name, first_name").query_all(conn).await
    }

    pub async fn active_during(semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM member WHERE email IN
             (SELECT member FROM active_semester WHERE semester = ?)",
             semester
        ).query_all(conn).await
    }

    pub fn full_name_inner(&self) -> String {
        format!("{} {}", self.preferred_name.as_deref().unwrap_or(&self.first_name), self.last_name)
    }

    pub async fn login_is_valid(email: &str, pass_hash: &str) -> Result<bool> {
        if let Some(hash) = sqlx::query!("SELECT pass_hash FROM member WHERE email = ?", email).query_optional(conn).await? {
            Password::from_hash(hash).verify(pass_hash)
        } else {
            false
        }
    }

    pub async fn is_active(&self, email: &str, conn: &DbConn) -> Result<bool> {
        let current_semester = Semester::current(conn).await?;
        Ok(self.semester(current_semester.name).await?.is_some())
    }

    pub async fn semester(&self, semester: &str, conn: &DbConn) -> Result<ActiveSemester> {
        ActiveSemester::for_member_during_semester(&self.email, semester, conn).ok_or_else(|| format!("{} was not active during {}", self.full_name())
    }

    def get_semester(semester_name)
      ActiveSemester.for_semester @email, semester_name
    end

    def get_semester!(semester_name)
      get_semester(semester_name) || raise "#{full_name} was not active during #{semester_name}"
    end

    def self.register(form)
      if CONN.query_one? "SELECT email FROM #{@@table_name} WHERE email = ?", form.email, as: String
        raise "Another member already has the email #{form.email}"
      end

      pass_hash = Password.create(form.pass_hash, cost: 10).to_s

      CONN.exec "INSERT INTO #{@@table_name} \
        (email, first_name, preferred_name, last_name, pass_hash, phone_number, \
         picture, passengers, location, on_campus, about, major, minor, hometown, \
         arrived_at_tech, gateway_drug, conflicts, dietary_restrictions)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        form.email, form.first_name, form.preferred_name, form.last_name, pass_hash,
        form.phone_number, form.picture, form.passengers, form.location, form.on_campus,
        form.about, form.major, form.minor, form.hometown, form.arrived_at_tech,
        form.gateway_drug, form.conflicts, form.dietary_restrictions

      Attendance.create_for_new_member form.email
    end

    def register_for_current_semester(form)
      ActiveSemester.create_for_member self, form, Semester.current

      CONN.exec "UPDATE #{@@table_name} \
        SET location = ?, on_campus = ?, conflicts = ?, dietary_restrictions = ? \
        WHERE email = ?", form.location, form.on_campus, form.conflicts,
        form.dietary_restrictions, @email

      @location = form.location
      @on_campus = form.on_campus
      @conflicts = form.conflicts
      @dietary_restrictions = form.dietary_restrictions

      Attendance.create_for_new_member @email
    end

    def update(form, as_self)
      if @email != form.email
        existing_email = CONN.query_one? "SELECT email FROM #{@@table_name} WHERE email = ?", form.email, as: String
        raise "Cannot change email to #{form.email}, as another member has that email" if existing_email
      end

      pass_hash = if new_hash = form.pass_hash
                    if as_self
                      Password.create(new_hash, cost: 10).to_s
                    else
                      raise "Only members themselves can change their own passwords"
                    end
                  else
                    @pass_hash
                  end

      CONN.exec "UPDATE #{@@table_name} SET \
        email = ?, first_name = ?, preferred_name = ?, last_name = ?, \
        phone_number = ?, picture = ?, passengers = ?, location = ?, \
        about = ?, major = ?, minor = ?, hometown = ?, arrived_at_tech = ?, \
        gateway_drug = ?, conflicts = ?, dietary_restrictions = ?, pass_hash = ?",
        form.email, form.first_name, form.preferred_name, form.last_name,
        form.phone_number, form.picture, form.passengers, form.location,
        form.about, form.major, form.minor, form.hometown, form.arrived_at_tech,
        form.gateway_drug, form.conflicts, form.dietary_restrictions, pass_hash

      @email = email
      @first_name = first_name
      @preferred_name = preferred_name
      @last_name = last_name
      @phone_number = phone_number
      @picture = picture
      @passengers = passengers
      @location = location
      @about = about
      @major = major
      @minor = minor
      @hometown = hometown
      @arrived_at_tech = arrived_at_tech
      @gateway_drug = gateway_drug
      @conflicts = conflicts
      @dietary_restrictions = dietary_restrictions

      ActiveSemester.update form.email, Semester.current, form.enrollment, form.section
    end

    def delete
      CONN.exec "DELETE FROM #{@@table_name} WHERE email = ?", @email
    end
  end
end

#[SimpleObject]
pub struct SectionType {
    /// The name of the section (Tenor, Baritone, etc.)
    pub name: String,
}

impl SectionType {
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM section_type ORDER BY name").query_all(conn).await
    }
}
