use db::models::*;
use db::schema::absence_request::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::*;

impl AbsenceRequest {
    pub fn load(given_user_email: &str, given_event_id: i32, conn: &MysqlConnection) -> Option<AbsenceRequest> {
        absence_request
            .filter(event.eq(&given_event_id))
            .filter(member.eq(&given_user_email))
            .first::<AbsenceRequest>(conn)
            .optional()
            .expect("error loading absence request")
    }

    // pub fn override_table_with_values(
    //     new_vals: &Vec<NewAbsenceRequest>,
    //     conn: &MysqlConnection,
    // ) -> QueryResult<()> {
    //     diesel::delete(absence_request).execute(conn)?;
    //     diesel::sql_query("ALTER SEQUENCE absence_requests_id_seq RESTART").execute(conn)?;
    //     diesel::insert_into(absence_request)
    //         .values(new_vals)
    //         .execute(conn)?;

    //     Ok(())
    // }
}

// impl From<AbsenceRequest> for NewAbsenceRequest {
//     fn from(absence_request: AbsenceRequest) -> Self {
//         NewAbsenceRequest {
//             event_id: absence_request.event_id,
//             user_email: absence_request.user_email,
//             reason: absence_request.reason,
//             status: absence_request.status,
//             time: absence_request.time,
//         }
//     }
// }
