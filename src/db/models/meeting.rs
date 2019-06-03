use db::models::*;
use db::schema::meetings::dsl::*;
use diesel::pg::PgConnection;
use diesel::*;

impl Meeting {
    pub fn load(given_meeting_id: i32, conn: &PgConnection) -> Result<Meeting, String> {
        meetings
            .filter(id.eq(given_meeting_id))
            .first::<Meeting>(conn)
            .optional()
            .expect("error loading meeting")
            .ok_or(format!(
                "no meeting exists with the id {}",
                given_meeting_id
            ))
    }

    pub fn load_all(conn: &PgConnection) -> Vec<Meeting> {
        meetings
            .order(time)
            .load::<Meeting>(conn)
            .expect("error loading meetings")
    }

    pub fn create(new_meeting: &NewMeeting, conn: &PgConnection) -> i32 {
        diesel::insert_into(meetings)
            .values(new_meeting)
            .execute(conn)
            .expect("error adding new meeting");

        meetings
            .filter(time.eq(&new_meeting.time))
            .first::<Meeting>(conn)
            .expect("error adding new meeting")
            .id
    }

    pub fn update(meeting_id: i32, updated_meeting: &NewMeeting, conn: &PgConnection) -> bool {
        diesel::update(meetings.find(meeting_id))
            .set(updated_meeting)
            .get_result::<Meeting>(conn)
            .is_ok()
    }
}

impl PublicJson for Meeting {}
