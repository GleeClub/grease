use db::models::*;
use db::traits::*;
use error::*;
use mysql::Conn;
use pinto::query_builder::{self, Join, Order};

impl AbsenceRequest {
    pub fn load(
        member: &str,
        event_id: i32,
        conn: &mut Conn,
    ) -> GreaseResult<Option<AbsenceRequest>> {
        Self::first_opt(
            &format!("event = {} AND member = '{}'", event_id, member),
            conn,
        )
    }

    pub fn load_all_for_this_semester(
        conn: &mut Conn,
    ) -> GreaseResult<Vec<(AbsenceRequest, Event)>> {
        let current_semester = Semester::load_current(conn)?;
        let query = query_builder::select(Self::table_name())
            .join(Event::table_name(), "event", "id", Join::Inner)
            .fields(AbsenceRequestEventRow::field_names())
            .filter(&format!("semester = '{}'", &current_semester.name))
            .order_by("`time`", Order::Desc)
            .build();

        crate::db::load::<AbsenceRequestEventRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
    }

    pub fn excused_for_event(member: &str, event_id: i32, conn: &mut Conn) -> GreaseResult<bool> {
        let query = query_builder::select(Self::table_name())
            .fields(&["state"])
            .filter(&format!("event = {} AND member = '{}'", event_id, member))
            .build();

        conn.first::<_, AbsenceRequestState>(query)
            .map_err(GreaseError::DbError)
            .map(|maybe_state| match maybe_state {
                Some(state) => state == AbsenceRequestState::Approved,
                None => false,
            })
    }

    pub fn create(member: &str, event_id: i32, reason: &str, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::insert(Self::table_name())
            .set("member", &format!("'{}'", member))
            .set("event", &event_id.to_string())
            .set("reason", &format!("'{}'", reason))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn approve(member: &str, event_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let request = AbsenceRequest::load(member, event_id, conn)?;
        let query = query_builder::update(Self::table_name())
            .filter(&format!("event = {} AND member = '{}'", event_id, member))
            .set("state", &format!("'{}'", AbsenceRequestState::Approved))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn deny(member: &str, event_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let request = AbsenceRequest::load(member, event_id, conn)?;
        let query = query_builder::update(Self::table_name())
            .filter(&format!("event = {} AND member = '{}'", event_id, member))
            .set("state", &format!("'{}'", AbsenceRequestState::Denied))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
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
