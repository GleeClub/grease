use db::models::*;
use db::schema::absence_request::dsl::*;
use db::schema::AbsenceRequestState;
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::*;

impl AbsenceRequest {
    pub fn load(
        given_user_email: &str,
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Option<AbsenceRequest>> {
        absence_request
            .filter(event.eq(&given_event_id).and(member.eq(&given_user_email)))
            .first::<AbsenceRequest>(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn excused_for_event(
        given_user_email: &str,
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<bool> {
        absence_request
            .filter(event.eq(&given_event_id).and(member.eq(&given_user_email)))
            .select(state)
            .first::<AbsenceRequestState>(conn)
            .optional()
            .map(|request| {
                request
                    .map(|req_state| req_state == AbsenceRequestState::Approved)
                    .unwrap_or(false)
            })
            .map_err(GreaseError::DbError)
    }
}
