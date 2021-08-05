#[derive(SimpleObject)]
pub struct Member {
    pub email:                String,
    pub first_name:           String,
    pub preferred_name:       Option<String>,
    pub last_name:            String,
    pub pass_hash:            String,
    pub phone_number:         String,
    pub picture:              Option<String>,
    pub passengers:           i32,
    pub location:             String,
    pub on_campus:            Option<bool>,
    pub about:                Option<String>,
    pub major:                Option<String>,
    pub minor:                Option<String>,
    pub hometown:             Option<String>,
    pub arrived_at_tech:      Option<Int32>,
    pub gateway_drug:         Option<String>,
    pub conflicts:            Option<String>,
    pub dietary_restrictions: Option<String>,
}

// @semesters : Hash(String, ActiveSemester)?

impl Member {
    pub async fn with_email_opt(email: &str, pool: &MySqlPool) -> Result<Option<Member>> {
        sqlx::query_as!(Member, "SELECT * FROM member WHERE email = ?", email).query_optional(pool).await.into()
    }

    pub async fn with_email(email: &str, pool: &MySqlPool) -> Result<Member> {
        Self::with_email_opt(email, pool).and_then(|res| res.ok_or_else(|| format!("No member with email {}", email)))
    }

    pub async fn with_token(token: &str, pool: &MySqlPool) -> Result<Member> {
        let session = Session::for_token(token, pool).await?;
        Self::with_email(&session.member, pool).await
    }

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY last_name, first_name", as: Member
    end

    def self.active_during(semester_name)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE email IN \
        (SELECT member FROM #{ActiveSemester.table_name} WHERE semester = ?)", semester_name, as: Member
    end

    def self.valid_login?(email, given_pass_hash)
      pass_hash = CONN.query_one? "SELECT pass_hash FROM #{@@table_name} \
        WHERE email = ?", email, as: String
      pass_hash && Password.new(raw_hash: pass_hash).verify(given_pass_hash)
    end

    def is_active?
      get_semester(Semester.current.name)
    end

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

    @[GraphQL::Field(description: "The member's email, which must be unique")]
    def email : String
      @email
    end

    @[GraphQL::Field(description: "The member's first name")]
    def first_name : String
      @first_name
    end

    @[GraphQL::Field(description: "The member's nick name")]
    def preferred_name : String?
      @preferred_name
    end

    @[GraphQL::Field(description: "The member's last name")]
    def last_name : String
      @last_name
    end

    @[GraphQL::Field(description: "The member's full name")]
    def full_name : String
      "#{@preferred_name || @first_name} #{@last_name}"
    end

    @[GraphQL::Field(description: "The member's phone number")]
    def phone_number : String
      @phone_number
    end

    @[GraphQL::Field(description: "An optional link to a profile picture for the member")]
    def picture : String?
      @picture
    end

    @[GraphQL::Field(description: "An optional link to a profile picture for the member")]
    def passengers : Int32
      @passengers
    end

    @[GraphQL::Field(description: "Where the member lives")]
    def location : String
      @location
    end

    @[GraphQL::Field(description: "Whether the member currently lives on campus (assumed false)")]
    def on_campus : Bool?
      @on_campus
    end

    @[GraphQL::Field(description: "The member's academic major")]
    def major : String?
      @major
    end

    @[GraphQL::Field(description: "The member's academic minor")]
    def minor : String?
      @minor
    end

    @[GraphQL::Field(description: "Where the member originally comes from")]
    def hometown : String?
      @hometown
    end

    @[GraphQL::Field(description: "What year the member arrived at Tech (e.g. 2012)")]
    def arrived_at_tech : Int32?
      @arrived_at_tech
    end

    @[GraphQL::Field(description: "What brought the member to Glee Club")]
    def gateway_drug : String?
      @gateway_drug
    end

    @[GraphQL::Field(description: "What conflicts during the week the member may have")]
    def conflicts : String?
      @conflicts
    end

    @[GraphQL::Field(description: "What dietary restrictions the member may have")]
    def dietary_restrictions : String?
      @dietary_restrictions
    end

    @[GraphQL::Field(description: "The name of the semester they were active during")]
    def semester : String?
      get_semester(Semester.current.name).try &.semester
    end

    @[GraphQL::Field(description: "The name of the semester they were active during")]
    def semesters(context : UserContext) : Array(Models::ActiveSemester)
      context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

      (ActiveSemester.all_for_member @email).sort_by &.semester
    end

    @[GraphQL::Field(description: "Whether they were in the class or the club")]
    def enrollment : Models::ActiveSemester::Enrollment?
      get_semester(Semester.current.name).try &.enrollment
    end

    @[GraphQL::Field(description: "Which section the member sang in")]
    def section : String?
      get_semester(Semester.current.name).try &.section
    end

    @[GraphQL::Field(description: "The officer positions currently held by the member")]
    def positions : Array(String)
      (Role.for_member @email).map &.name
    end

    @[GraphQL::Field(description: "The permissions held currently by the member")]
    def permissions : Array(Models::MemberPermission)
      MemberPermission.for_member @email
    end

    @[GraphQL::Field(description: "The grades for the member in the given semester (default the current semester)")]
    def grades(context : UserContext) : Models::Grades
      context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

      Grades.for_member self, Semester.current
    end

    @[GraphQL::Field(description: "All of the member's transactions for their entire time in Glee Club")]
    def transactions(context : UserContext) : Array(Models::ClubTransaction)
      context.able_to! Permissions::VIEW_USER_PRIVATE_DETAILS unless @email == context.user!.email

      ClubTransaction.for_member_during_semester @email, Semester.current.name
    end
  end

  class SectionType
    class_getter table_name = "section_type"

    DB.mapping({
      name: String,
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY name", as: SectionType
    end
  end
end
