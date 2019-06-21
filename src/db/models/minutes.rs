use crate::error::*;
use db::models::MeetingMinutes;
use db::traits::*;
use mysql::Conn;
use pinto::query_builder::{self, Order};
use serde::Deserialize;
use serde_json::{json, Value};

impl MeetingMinutes {
    pub fn load(given_meeting_id: i32, conn: &mut Conn) -> GreaseResult<MeetingMinutes> {
        Self::first(
            &format!("id = {}", given_meeting_id),
            conn,
            format!("No meeting minutes with id {}", given_meeting_id),
        )
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<MeetingMinutes>> {
        Self::query_all_in_order(vec![("date, name", Order::Desc)], conn)
    }

    pub fn create(new_meeting: &NewMeetingMinutes, conn: &mut Conn) -> GreaseResult<i32> {
        new_meeting.insert_returning_id("id", conn)
    }

    pub fn update(
        meeting_id: i32,
        updated_meeting: &NewMeetingMinutes,
        conn: &mut Conn,
    ) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", meeting_id))
            .set("name", &updated_meeting.name)
            .set(
                "public",
                &updated_meeting
                    .public
                    .as_ref()
                    .unwrap_or(&"NULL".to_owned()),
            )
            .set(
                "private",
                &updated_meeting
                    .private
                    .as_ref()
                    .unwrap_or(&"NULL".to_owned()),
            )
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(meeting_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("id = {}", meeting_id))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
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
