use db::*;
use error::*;
use pinto::query_builder::*;
use serde::Deserialize;
use serde_json::{json, Value};

impl MeetingMinutes {
    pub fn load<C: Connection>(meeting_id: i32, conn: &mut C) -> GreaseResult<MeetingMinutes> {
        conn.first(&Self::filter(&format!("id = {}", meeting_id)), format!("No meeting minutes with id {}.", meeting_id))
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<MeetingMinutes>> {
        conn.load(&Self::select_all_in_order("date, name", Order::Desc))
    }

    pub fn create<C: Connection>(new_meeting: &NewMeetingMinutes, conn: &mut C) -> GreaseResult<i32> {
        new_meeting.insert_returning_id(conn)
    }

    pub fn update<C: Connection>(
        meeting_id: i32,
        updated_meeting: &NewMeetingMinutes,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.update(
            &Update::new(Self::table_name())
                .filter(&format!("id = {}", meeting_id))
                .set("name", &to_value(&updated_meeting.name))
                .set("public", &to_value(&updated_meeting.public))
                .set("private", &to_value(&updated_meeting.private)),
            format!("No meeting minutes with id {}.", meeting_id),
        )
    }

    pub fn delete<C: Connection>(meeting_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.delete(&Delete::new(Self::table_name()).filter(&format!("id = {}", meeting_id)), format!("No meeting minutes with id {}.", meeting_id))
    }

    pub fn to_json(&self, can_view_private: bool) -> Value {
        let mut meeting = json!({
            "id": &self.id,
            "name": &self.name,
            "date": &self.date,
            "public": &self.public,
        });
        if can_view_private {
            meeting["private"] = json!(self.private);
        }

        meeting
    }
}

#[derive(
    grease_derive::TableName, grease_derive::Insertable, Deserialize, grease_derive::Extract,
)]
#[table_name = "minutes"]
pub struct NewMeetingMinutes {
    pub name: String,
    pub private: Option<String>,
    pub public: Option<String>,
}
