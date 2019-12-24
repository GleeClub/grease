use db::schema::minutes::dsl::*;
use db::{MeetingMinutes, NewMeetingMinutes, UpdatedMeetingMinutes};
use diesel::Connection;
use error::*;
use serde_json::{json, Value};

impl MeetingMinutes {
    pub fn load<C: Connection>(meeting_id: i32, conn: &mut C) -> GreaseResult<MeetingMinutes> {
        minutes
            .filter(id.eq(meeting_id))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
        // format!("No meeting minutes with id {}.", meeting_id),
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<MeetingMinutes>> {
        minutes
            .order_by((date.desc(), name.desc()))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create<C: Connection>(
        new_meeting: &NewMeetingMinutes,
        conn: &mut C,
    ) -> GreaseResult<i32> {
        conn.transaction(|| {
            diesel::insert_into(minutes)
                .values(new_meeting)
                .execute(conn)?;

            minutes.order_by(id.desc()).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn update<C: Connection>(
        meeting_id: i32,
        updated_meeting: &UpdatedMeetingMinutes,
        conn: &mut C,
    ) -> GreaseResult<()> {
        diesel::update(minutes.filter(id.eq(meeting_id)))
            .set(updated_meeting)
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No meeting minutes with id {}.", meeting_id),
    }

    pub fn delete<C: Connection>(meeting_id: i32, conn: &mut C) -> GreaseResult<()> {
        diesel::delete(minutes.filter(id.eq(meeting_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No meeting minutes with id {}.", meeting_id),
    }

    pub fn to_json(&self, can_view_private: bool) -> Value {
        json!({
            "id": &self.id,
            "name": &self.name,
            "date": &self.date,
            "public": &self.public,
            "private": &self.private.filter(|_| can_view_private)
        })
    }
}
