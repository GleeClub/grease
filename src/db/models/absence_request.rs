use db::schema::{event, AbsenceRequestState};
use db::{AbsenceRequest, Event, Semester};
use diesel::prelude::*;
use error::*;

impl AbsenceRequest {
    pub fn load(
        given_member: &str,
        event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<AbsenceRequest>> {
        use db::schema::absence_request::dsl::{absence_request, event, member};

        absence_request
            .filter(event.eq(event_id).and(member.eq(given_member)))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_this_semester(
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<(AbsenceRequest, Event)>> {
        use db::schema::absence_request::{self, dsl::time};

        let current_semester = Semester::load_current(conn)?;
        absence_request::table
            .inner_join(event::table)
            .filter(event::dsl::semester.eq(current_semester.name))
            .order_by(time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn excused_for_event(
        given_member: &str,
        event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<bool> {
        Self::load(given_member, event_id, conn).map(|request| {
            request
                .map(|r| r.state == AbsenceRequestState::Approved)
                .unwrap_or(false)
        })
    }

    pub fn create(
        given_member: &str,
        event_id: i32,
        given_reason: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::absence_request::dsl::*;

        diesel::insert_into(absence_request)
            .values((
                event.eq(event_id),
                member.eq(given_member),
                reason.eq(given_reason),
            ))
            .execute(conn)?;

        Ok(())
    }

    pub fn approve(member: &str, event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        Self::set_request_state(member, event_id, AbsenceRequestState::Approved, conn)
    }

    pub fn deny(member: &str, event_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        Self::set_request_state(member, event_id, AbsenceRequestState::Denied, conn)
    }

    fn set_request_state(
        given_member: &str,
        event_id: i32,
        given_state: AbsenceRequestState,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::absence_request::dsl::*;

        let _request = AbsenceRequest::load(given_member, event_id, conn)?.ok_or(
            GreaseError::BadRequest(format!(
                "No absence request for member {} at event with id {}.",
                given_member, event_id
            )),
        )?;

        diesel::update(absence_request.filter(event.eq(event_id).and(member.eq(given_member))))
            .set(state.eq(given_state))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }
}
