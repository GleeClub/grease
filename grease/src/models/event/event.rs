use async_graphql::{Enum, SimpleObject, InputObject, ComplexObject};
use crate::db_conn::DbConn;

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
    /// When members are probably going to be released
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
        let conn = ctx.data_unchecked::<DbConn>();
        Gig::for_event(self.id, conn)
    }

    /// The attendance for the current user at this event
    pub async fn user_attendance(&self, ctx: &Context<'_>) -> Result<Attendance> {
        let conn = ctx.data_unchecked::<DbConn>();
        let user = unimplemented!("get member");

        Attendance::for_member_at_event(user.email, self.id, conn).await
    }

    /// The attendance for a specific member at this event
    pub async fn attendance(&self, ctx: &Context<'_>, member: String) -> Result<Attendance> {
        let conn = ctx.data_unchecked::<DbConn>();
        Attendance::for_member_at_event(member, self.id, conn).await
    }

    pub async fn all_attendance(&self, ctx: &Context<'_>) -> Result<Vec<Attendance>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Attendance::for_event(self.id, conn).await
    }

    pub async fn carpools(&self, ctx: &Context<'_>) -> Result<Vec<Carpool>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Carpool::for_event(self.id, conn).await
    }

    pub async fn setlist(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Song::setlist_for_event(self.id, conn).await
    }
}

impl Event {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| format!("No event with id {}", id))
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
        self.event_type == Event::TUTTI_GIG || self.event_type == Event::VOLUNTEER_GIG
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
            // TODO: update event types to constants
            Some(format!("You cannot RSVP for {} events.", bad_type))
        } else {
            None
        }
    }

    pub async fn create(new_event: NewEvent, from_request: Option<GigRequest>,conn: &DbConn) -> Result<isize> {
        if let Some(release_time) = new_event.event.release_time {
            if release_time <= new_event.event.call_time {
                return Err("Release time must be after call time".into());
            }
        }

        // TODO: redo with a `match`
        let (period, repeat_until) = if let Some(repeat) = new_event.repeat {
            let repeat_until = if repeat.period == Period::No {
                new_event.event.call_time
            } else {
                repeat.repeat_until.ok_or("Must supply a repeat until time if repeat is supplied")
            };

            (repeat.period, repeat.repeat_until)
        } else {
            return Err("Must supply a repeat for new events");
        };

        let call_and_release_times = Self::repeat_event_times(
            new_event.event.call_time, new_event.event.release_time, period, repeat_until);
        if call_and_release_times.is_empty() {
            return Err("The repeat setting would render no events");
        }

        for (call_time, release_time) in call_and_release_times.iter() {
            sqlx::query!(
                "INSERT INTO event
                     (name, semester, `type`, call_time, release_time, points,
                      comments, location, gig_count, default_attend)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                 new_event.event.name, new_event.event.semester, new_event.event.r#type,
                 call_time, release_time, new_event.event.points, new_event.event.comments,
                 new_event.event.location, new_event.event.gig_count, new_event.event.default_attend
            ).query(conn).await?;
        }

        let new_ids = sqlx::query!(
            "SELECT id FROM event ORDER BY id DESC LIMIT ?",
            call_and_release_times.len())
                .query_all(conn).await?;
        for new_id in new_ids.iter() {
            Attendance::create_for_new_event(new_id, conn).await?;
        }

        if let Some(gig) = new_event.gig.or_else(|| from_request.map(|request| request.build_new_gig())) {
            for new_id in new_ids {
                sqlx::query!(
                    "INSERT INTO gig
                        (event, performance_time, uniform, contact_name, contact_email,
                         contact_phone, price, `public`, summary, description)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                     new_id, new_event.gig.performance_time, new_event.gig.uniform,
                     new_event.gig.contact_name, new_event.gig.contact_email,
                     new_event.gig.contact_phone, new_event.gig.price, new_event.gig_public,
                     new_event.gig.summary, new_event.gig.description
                ).query_all(conn).await
            }
        }

        if let Some(request) = from_request {
            GigRequest::set_status(request.id, GigRequestStatus::Accepted, conn).await?;
        }

        new_ids.last().ok_or_else(|| "Failed to find latest event ID")
    }

    pub async fn update(id: isize, update: EventUpdate, conn: &DbConn) -> Result<()> {
        let event = Self::with_id(id, conn).await?;

        sqlx::query!(
            "UPDATE event SET name = ?, semester = ?, `type` = ?, call_time = ?, release_time = ?,
                points = ?, comments = ?, location = ?, gig_count = ?, default_attend = ?
             WHERE id = ?",
            update.event.name, update.event.semester, update.event.r#type, update.event.call_time,
            update.event.release_time, update.event.points, update.event.comments, update.event.location,
            update.event.gig_count, update.event.default_attend, id
        ).query(conn).await?;

        if event.gig.is_some() {
            if let Some(gig) = update.gig {
                sqlx::query!(
                    "UPDATE gig SET performance_time = ?, uniform = ?, contact_name = ?, contact_email = ?,
                     contact_phone = ?, price = ?, public = ?, summary = ?, description = ?
                     WHERE event = ?", gig.performance_time, gig.uniform, gig.contact_name, gig.contact_email,
                    gig.contact_phone, gig.price, gig.public, gig.summary, gig.description, id).query(conn).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(id: isize, conn: &DbConn) -> Result<()> {
        // TODO: verify exists?
        Event::with_id(id, conn).await?;

        sqlx::query!("DELETE FROM event WHERE id = ?", id).query(conn).await
    }

    fn repeat_event_times(call_time: NaiveDateTime, release_time: NaiveDateTime, period: Period, repeat_until: NaiveDateTime) -> Vec<(NaiveDateTime, NaiveDateTime)> {
        // def self.repeat_event_times(call_time, release_time, period, repeat_until)
        //   pairs = [] of {Time, Time?}

        //   while call_time < repeat_until
        //     pairs << {call_time, release_time}

        //     case period
        //     when Input::Period::NO
        //       break
        //     when Input::Period::DAILY
        //       call_time = call_time.shift days: 1
        //       release_time = release_time.try &.shift days: 1
        //     when Input::Period::WEEKLY
        //       call_time = call_time.shift weeks: 1
        //       release_time = release_time.try &.shift weeks: 1
        //     when Input::Period::BIWEEKLY
        //       call_time = call_time.shift weeks: 2
        //       release_time = release_time.try &.shift weeks: 2
        //     when Input::Period::MONTHLY
        //       call_time = call_time.shift months: 1
        //       release_time = release_time.try &.shift months: 1
        //     when Input::Period::YEARLY
        //       call_time = call_time.shift years: 1
        //       release_time = release_time.try &.shift years: 1
        //     end
        //   end

        //   pairs
        // end
    }
}

#[derive(InputObject)]
pub struct NewEventPeriod {
    pub period: Period,
    pub repeat_until: Option<NaiveDateTime>,
}
