use db::*;
use error::*;
use pinto::query_builder::*;
use chrono::NaiveDateTime;

impl AbsenceRequest {
    pub fn load<C: Connection>(
        member: &str,
        event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<Option<AbsenceRequest>> {
        conn.first_opt(&Self::filter(&format!("event = {} AND member = '{}'", event_id, member)))
    }

    pub fn load_all_for_this_semester<C: Connection>(
        conn: &mut C,
    ) -> GreaseResult<Vec<(AbsenceRequest, Event)>> {
        let current_semester = Semester::load_current(conn)?;
        conn.load_as::<AbsenceRequestEventRow, _>(
            Select::new(Self::table_name())
                .join(Event::table_name(), "event", "id", Join::Inner)
                .fields(AbsenceRequestEventRow::field_names())
                .filter(&format!("semester = '{}'", &current_semester.name))
                .order_by("`time`", Order::Desc)
        )
    }

    pub fn excused_for_event<C: Connection>(member: &str, event_id: i32, conn: &mut C) -> GreaseResult<bool> {
        conn.first_opt(&Self::filter(&format!("event = {} AND member = '{}'", event_id, member))).map(|request: Option<AbsenceRequest>| request.map(|r| r.state == AbsenceRequestState::Approved).unwrap_or(false))
    }

    pub fn create<C: Connection>(member: &str, event_id: i32, reason: &str, conn: &mut C) -> GreaseResult<()> {
        conn.insert(
            Insert::new(Self::table_name())
                .set("member", &format!("'{}'", member))
                .set("event", &event_id.to_string())
                .set("reason", &format!("'{}'", reason))
        )
    }

    pub fn approve<C: Connection>(member: &str, event_id: i32, conn: &mut C) -> GreaseResult<()> {
        let _request = AbsenceRequest::load(member, event_id, conn)?;

        conn.update_opt(
            Update::new(Self::table_name())
                .filter(&format!("event = {} AND member = '{}'", event_id, member))
                .set("state", &format!("'{}'", AbsenceRequestState::Approved))
        )
    }

    pub fn deny<C: Connection>(member: &str, event_id: i32, conn: &mut C) -> GreaseResult<()> {
        let _request = AbsenceRequest::load(member, event_id, conn)?;

        conn.update_opt(
            Update::new(Self::table_name())
                .filter(&format!("event = {} AND member = '{}'", event_id, member))
                .set("state", &format!("'{}'", AbsenceRequestState::Denied))
        )
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
struct AbsenceRequestEventRow {
    // absence request fields
    pub member: String,
    pub event: i32,
    pub time: NaiveDateTime,
    pub reason: String,
    pub state: AbsenceRequestState,
    // event fields
    pub id: i32,
    pub name: String,
    pub semester: String,
    #[rename = "type"]
    pub type_: String,
    pub call_time: NaiveDateTime,
    pub release_time: Option<NaiveDateTime>,
    pub points: i32,
    pub comments: Option<String>,
    pub location: Option<String>,
    pub gig_count: bool,
    pub default_attend: bool,
    pub section: Option<String>,
}

impl Into<(AbsenceRequest, Event)> for AbsenceRequestEventRow {
    fn into(self) -> (AbsenceRequest, Event) {
        (
            AbsenceRequest {
                member: self.member,
                event: self.event,
                time: self.time,
                reason: self.reason,
                state: self.state,
            },
            Event {
                id: self.id,
                name: self.name,
                semester: self.semester,
                type_: self.type_,
                call_time: self.call_time,
                release_time: self.release_time,
                points: self.points,
                comments: self.comments,
                location: self.location,
                gig_count: self.gig_count,
                default_attend: self.default_attend,
                section: self.section,
            },
        )
    }
}
