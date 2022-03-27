use async_graphql::{Enum, SimpleObject, ComplexObject};

#[derive(Enum)]
pub enum Period {
    No,
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Yearly,
}

#[derive(SimpleObject)]
pub struct EventType {
    /// The name of the type of event
    pub name: String,
    /// The amount of points this event is normally worth
    pub weight: isize,
}

impl EventType {
    pub const REHEARSAL: &str     = "Rehearsal";
    pub const SECTIONAL: &str     = "Sectional";
    pub const VOLUNTEER_GIG: &str = "Volunteer Gig";
    pub const TUTTI_GIG: &str     = "Tutti Gig";
    pub const OMBUDS: &str        = "Ombuds";
    pub const OTHER: &str         = "Other";

    pub fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM event_type ORDER BY name")
            .query_all(conn).await
    }
}

#[derive(SimpleObject)]
pub struct Event {
    /// The ID of the event
    pub id: isize,
    /// The name of the event
    pub name: String,
    /// The name of the semester this event belongs to
    pub semester: String,
    /// The type of the event (see EventType)
    pub r#type: String,
    /// When members are expected to arrive to the event
    pub call_time: NaiveDateTime,
    /// When members are provably going to be released
    pub release_time: Option<NaiveDateTime>,
    /// How many points attendance of this event is worth
    pub points: isize,
    /// General information or details about this event
    pub comments: Option<String>,
    /// Where this event will be held
    pub location: Option<String>,
    /// Whether this event counts toward the volunteer gig count for the semester
    pub gig_count: bool,
    /// Whether members are assumed to attend (we assume as much for most events)
    pub default_attend: bool,
}

#[ComplexObject]
impl Event {
    /// The gig for this event, if it is a gig
    pub async fn gig(&self, ctx: &Context<'_>) -> Result<Option<Gig>> {
        Gig::for_event(self.id, ctx.data_unchecked::<DbConn>())
    }

    /// The attendance for the current user at this event
    pub async fn user_attendance(&self, ctx: &Context<'_>) -> Result<Attendance> {
        let conn = ctx.data_unchecked::<DbConn>();
        let user = unimplemented!("get member");

        Attendance::for_member_at_event(user.email, self.id, conn).await
    }

    @[GraphQL::Field]
    def attendance(member : String) : Models::Attendance
      Attendance.for_member_at_event! member, @id
    end

    @[GraphQL::Field]
    def all_attendance : Array(Models::Attendance)
      Attendance.for_event @id
    end

    @[GraphQL::Field]
    def carpools : Array(Models::Carpool)
      Carpool.for_event @id
    end

    @[GraphQL::Field]
    def setlist : Array(Models::Song)
      Song.setlist_for_event @id
    end
}

impl Event {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| anyhow::anyhow!("No event with id {}", id))
    }

    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM event WHERE id = ?", id)
            .query_optional(conn).await
    }

    pub async fn for_semester(semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        Semester::verify_exists(semester, conn).await?;

        sqlx::query_as!(Self, "SELECT * FROM event WHERE semester = ?", semester)
            .query_all(conn)
            .await
    }

    pub fn is_gig(&self) -> bool {
        self.event_type
    }

    pub fn ensure_no_rsvp_issue(&self,member: &Member, attendance: &Option<Attendance>) -> Result<()> {
        if let Some(rsvp_issue) = self.rsvp_issue_for(member, attendance) {
            Err(rsvp_issue)
        } else {
            Ok(())
        }
    }

    pub fn rsvp_issue_for(&self, member: &Member, attendance: &Option<Attendance>) -> Option<String> {
        if !member.is_active {
            Some("Member must be active to RSVP to events.".to_owned())
        } else if !attendance.map(|a| a.should_attend).unwrap_or(true) {
            None
        } else if Local::now().naive_local() + Duration::days(1) > self.call_time {
            Some("Responses are closed for this event.".to_owned())
        } else if let Some(bad_type) = ["Tutti Gig", "Sectional", "Rehearsal"]
            .contains(&self.type_)
        {
            Some(format!("You cannot RSVP for {} events.", bad_type))
        } else {
            None
        }
    }

    pub async fn create(new_event: NewEvent, from_request: Option<GigRequest>,conn: &DbConn) -> Result<isize> {
        if let Some(release_time) = new_event.event.release_time {
            if release_time < new_event.event.call_time {
                return Err("Release time must be after call time".into());
            }
        }

    }
}

    def self.create(form, from_request = nil)
      if release_time = form.event.release_time
        raise "Release time must be after call time" if release_time <= form.event.call_time
      end

      period, repeat_until = if r = form.repeat
                               {r.period, r.repeat_until}
                             else
                               raise "Must supply a repeat for new events"
                             end
      repeat_until = if period == Input::Period::NO
                       form.event.call_time
                     else
                       repeat_until || raise "Must supply a repeat until time if repeat is supplied"
                     end

      e = form.event
      call_and_release_times = repeat_event_times e.call_time, e.release_time, period, repeat_until
      raise "The repeat setting would render no events" if call_and_release_times.empty?

      call_and_release_times.each do |(call_time, release_time)|
        CONN.exec "INSERT INTO #{@@table_name} \
          (name, semester, type, call_time, release_time, points, \
           comments, location, gig_count, default_attend)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
          e.name, e.semester, e.type, call_time, release_time, e.points,
          e.comments, e.location, e.gig_count, e.default_attend
      end

      new_ids = CONN.query_all "SELECT id FROM #{@@table_name} ORDER BY id DESC LIMIT ?",
        call_and_release_times.size, as: Int32
      new_ids.each { |id| Attendance.create_for_new_event id }

      if g = (form.gig || from_request.try &.build_new_gig)
        new_ids.each do |id|
          CONN.exec "INSERT INTO #{Gig.table_name} \
            (event, performance_time, uniform, contact_name, contact_email, \
             contact_phone, price, public, summary, description) \
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id, g.performance_time, g.uniform, g.contact_name, g.contact_email,
            g.contact_phone, g.price, g.public, g.summary, g.description
        end
      end

      from_request.set_status GigRequest::Status::ACCEPTED if from_request

      new_ids.first
    end

    def self.repeat_event_times(call_time, release_time, period, repeat_until)
      pairs = [] of {Time, Time?}

      while call_time < repeat_until
        pairs << {call_time, release_time}

        case period
        when Input::Period::NO
          break
        when Input::Period::DAILY
          call_time = call_time.shift days: 1
          release_time = release_time.try &.shift days: 1
        when Input::Period::WEEKLY
          call_time = call_time.shift weeks: 1
          release_time = release_time.try &.shift weeks: 1
        when Input::Period::BIWEEKLY
          call_time = call_time.shift weeks: 2
          release_time = release_time.try &.shift weeks: 2
        when Input::Period::MONTHLY
          call_time = call_time.shift months: 1
          release_time = release_time.try &.shift months: 1
        when Input::Period::YEARLY
          call_time = call_time.shift years: 1
          release_time = release_time.try &.shift years: 1
        end
      end

      pairs
    end

    def self.update(id, form)
      event = Event.with_id! id

      raise "Gig fields must be present when updating gig events" if event.gig && !form.gig

      e = form.event
      CONN.exec "UPDATE #{@@table_name} SET \
        name = ?, semester = ?, type = ?, call_time = ?, release_time = ?, points = ?, \
        comments = ?, location = ?, gig_count = ?. default_attend = ? \
        WHERE id = ?",
        e.name, e.semester, e.type, e.call_time, e.release_time, e.points,
        e.comments, e.location, e.gig_count, e.default_attend, id

      if event.gig
        if g = form.gig
          CONN.exec "UPDATE #{Gig.table_name} SET \
            performance_time = ?, uniform = ?, contact_name = ?, contact_email = ?, \
            contact_phone = ?, price = ?, public = ?, summary = ?, description = ? \
            WHERE event = ?",
            g.performance_time, g.uniform, g.contact_name, g.contact_email,
            g.contact_phone, g.price, g.public, g.summary, g.description, id
        end
      end
    end

    def self.delete(id)
      Event.with_id! id

      CONN.exec "DELETE FROM #{@@table_name} WHERE id = ?", id
    end
  end
end
