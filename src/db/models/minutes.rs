use db::schema::minutes::dsl::*;
use db::{MeetingMinutes, NewMeetingMinutes, UpdatedMeetingMinutes};
use diesel::prelude::*;
use error::*;
use serde_json::{json, Value};

impl MeetingMinutes {
    pub fn load(meeting_id: i32, conn: &MysqlConnection) -> GreaseResult<MeetingMinutes> {
        minutes
            .filter(id.eq(meeting_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "No meeting minutes with id {}.",
                meeting_id
            )))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<MeetingMinutes>> {
        minutes
            .order_by((date.desc(), name.desc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(new_meeting: &NewMeetingMinutes, conn: &MysqlConnection) -> GreaseResult<i32> {
        conn.transaction(|| {
            diesel::insert_into(minutes)
                .values((
                    name.eq(&new_meeting.name),
                    date.eq(chrono::Local::today().naive_local()),
                ))
                .execute(conn)?;

            minutes.select(id).order_by(id.desc()).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn update(
        meeting_id: i32,
        updated_meeting: &UpdatedMeetingMinutes,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        diesel::update(minutes.filter(id.eq(meeting_id)))
            .set(updated_meeting)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
        // format!("No meeting minutes with id {}.", meeting_id),
    }

    pub fn delete(meeting_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        diesel::delete(minutes.filter(id.eq(meeting_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
        // format!("No meeting minutes with id {}.", meeting_id),
    }

    pub fn to_json(&self, can_view_private: bool) -> Value {
        json!({
            "id": &self.id,
            "name": &self.name,
            "date": &self.date.and_hms(0, 0, 0).timestamp() * 1000,
            "public": &self.public,
            "private": &self.private.as_ref().filter(|_| can_view_private)
        })
    }
}
