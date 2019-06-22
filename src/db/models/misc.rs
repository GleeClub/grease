use crate::db::load;
use crate::db::traits::*;
use crate::error::{GreaseError, GreaseResult};
use crate::util::random_base64;
use db::models::*;
use mysql::prelude::ToValue;
use mysql::Conn;
use pinto::query_builder::{self, Join, Order};

impl GoogleDoc {
    pub fn load(doc_name: &str, conn: &mut Conn) -> GreaseResult<GoogleDoc> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("name = '{}'", doc_name))
            .build();

        match conn.first(query) {
            Ok(Some(doc)) => Ok(doc),
            Ok(None) => Err(GreaseError::BadRequest(format!(
                "no google doc named {}",
                doc_name
            ))),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<GoogleDoc>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .order_by("name", Order::Asc)
            .build();

        load(&query, conn)
    }

    pub fn insert(new_doc: &GoogleDoc, conn: &mut Conn) -> GreaseResult<()> {
        new_doc.insert(conn)
    }

    pub fn update(old_name: &str, changed_doc: &GoogleDoc, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("name = '{}'", old_name))
            .set("name", &changed_doc.name.to_value().as_sql(true))
            .set("url", &changed_doc.url.to_value().as_sql(true))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(name: &str, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .build();

        conn.query(query).map_err(GreaseError::DbError)?;
        Ok(())
    }
}

impl Announcement {
    pub fn load(given_id: i32, conn: &mut Conn) -> GreaseResult<Announcement> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("id = {}", given_id))
            .build();

        match conn.first(query) {
            Ok(Some(announcement)) => Ok(announcement),
            Ok(None) => Err(GreaseError::BadRequest(format!(
                "no announcement with id {}",
                given_id
            ))),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn insert(
        new_content: &str,
        given_member: &str,
        given_semester: &str,
        conn: &mut Conn,
    ) -> GreaseResult<i32> {
        let insert_query = query_builder::insert(Self::table_name())
            .set("member", given_member)
            .set("semester", given_semester)
            .set("content", new_content)
            .build();
        conn.query(insert_query).map_err(GreaseError::DbError)?;

        let id_query = query_builder::select(Self::table_name())
            .fields(&["id"])
            .order_by("id", Order::Desc)
            .build();

        match conn.first(id_query) {
            Ok(Some(id)) => Ok(id),
            Ok(None) => Err(GreaseError::ServerError(
                "no announcement was actually inserted".to_owned(),
            )),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<Announcement>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .order_by("time", Order::Desc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn load_all_for_semester(
        given_semester: &str,
        conn: &mut Conn,
    ) -> GreaseResult<Vec<Announcement>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("semester = '{}'", given_semester))
            .filter("archived = false")
            .order_by("time", Order::Desc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn archive(given_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", given_id))
            .set("archived", "true")
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl Uniform {
    pub fn load(name: &str, conn: &mut Conn) -> GreaseResult<Uniform> {
        Self::first(
            &format!("name = '{}'", name),
            conn,
            format!("no uniform with name {}", name),
        )
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<Uniform>> {
        Self::query_all_in_order(vec![("name", Order::Asc)], conn)
    }

    pub fn update(old_name: &str, updated: &Uniform, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("name = '{}'", old_name))
            .set("name", &updated.name)
            .set("color", &updated.color.to_value().as_sql(false))
            .set("description", &updated.description.to_value().as_sql(false))
            .build();

        conn.query(query).map_err(GreaseError::DbError)?;
        Ok(())
    }

    pub fn delete(name: &str, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .build();

        conn.query(query).map_err(GreaseError::DbError)?;
        Ok(())
    }

    pub fn validate(&self) -> GreaseResult<()> {
        let regex = regex::Regex::new(r"^#\w{3}$").unwrap();

        // if color string is invalid
        if self
            .color
            .as_ref()
            .map(|color| !regex.is_match(&color))
            .unwrap_or(false)
        {
            Err(GreaseError::BadRequest(
                "uniform colors must be in the format '#XXX', where X is a hexadecimal number"
                    .to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

impl MediaType {
    pub fn load(type_name: &str, conn: &mut Conn) -> GreaseResult<MediaType> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("name = '{}'", type_name))
            .build();

        match conn.first(query) {
            Ok(Some(type_)) => Ok(type_),
            Ok(None) => Err(GreaseError::BadRequest(format!(
                "no media type named {}",
                type_name
            ))),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<MediaType>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .order_by("`order`", Order::Desc)
            .build();

        crate::db::load(&query, conn)
    }
}

impl Variable {
    pub fn load(given_key: &str, conn: &mut Conn) -> GreaseResult<Option<String>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("`key` = {}", given_key.to_value().as_sql(true)))
            .build();

        match conn.first(query) {
            Ok(maybe_var) => Ok(maybe_var.map(|var: Variable| var.value)),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn set(
        given_key: String,
        new_value: String,
        conn: &mut Conn,
    ) -> GreaseResult<Option<String>> {
        if let Some(val) = Variable::load(&given_key, conn)? {
            let query = query_builder::update(Self::table_name())
                .filter(&format!("`key` = '{}'", &given_key))
                .set("`key`", &given_key)
                .set("value", &new_value)
                .build();

            match conn.query(query) {
                Ok(_result) => Ok(Some(val)),
                Err(error) => Err(GreaseError::DbError(error)),
            }
        } else {
            let new_var = Variable {
                key: given_key,
                value: new_value,
            };

            new_var.insert(conn).map(|_| None)
        }
    }

    pub fn unset(given_key: &str, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("`key` = '{}'", given_key))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl Session {
    pub fn load(given_email: &str, conn: &mut Conn) -> GreaseResult<Option<Session>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("member = '{}'", given_email))
            .build();

        conn.first(query).map_err(GreaseError::DbError)
    }

    pub fn delete(given_email: &str, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::delete(Self::table_name())
            .filter(&format!("member = '{}'", given_email))
            .build();

        match conn.query(query) {
            Ok(_result) => Ok(()),
            Err(error) => Err(GreaseError::DbError(error)),
        }
    }

    pub fn generate(given_email: &str, conn: &mut Conn) -> GreaseResult<String> {
        let new_session = Session {
            member: given_email.to_owned(),
            key: random_base64(32)?,
        };

        new_session.insert(conn).map(|_| new_session.key)
    }
}

impl GigSong {
    pub fn load_for_event(event_id: i32, conn: &mut Conn) -> GreaseResult<Vec<GigSong>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("event = {}", event_id))
            .order_by("`order`", Order::Asc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn update_for_event(
        event_id: i32,
        updated_setlist: Vec<NewGigSong>,
        conn: &mut Conn,
    ) -> GreaseResult<()> {
        let gig_songs = updated_setlist
            .into_iter()
            .enumerate()
            .map(|(index, gig_song)| GigSong {
                event: event_id,
                song: gig_song.song,
                order: index as i32 + 1,
            })
            .collect::<Vec<GigSong>>();

        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;
        let delete_query = query_builder::delete(Self::table_name())
            .filter(&format!("event = {}", event_id))
            .build();
        transaction
            .query(delete_query)
            .map_err(GreaseError::DbError)?;

        for gig_song in gig_songs {
            gig_song.insert(&mut transaction)?;
        }
        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl Todo {
    pub fn load(todo_id: i32, conn: &mut Conn) -> GreaseResult<Todo> {
        Todo::first(
            &format!("id = {}", todo_id),
            conn,
            format!("no todo with id {}", todo_id),
        )
    }

    pub fn load_all_for_member(member: &str, conn: &mut Conn) -> GreaseResult<Vec<Todo>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("member = '{}'", member))
            .filter("completed = false")
            .build();

        crate::db::load(&query, conn)
    }

    pub fn create(new_todo: NewTodo, conn: &mut Conn) -> GreaseResult<()> {
        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;
        for member in new_todo.members {
            let query = query_builder::insert(Self::table_name())
                .set("`text`", &format!("'{}'", new_todo.text))
                .set("member", &format!("'{}'", member))
                .build();
            transaction.query(&query).map_err(GreaseError::DbError)?;
        }
        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn mark_complete(todo_id: i32, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("id = {}", todo_id))
            .set("completed", "true")
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl RolePermission {
    pub fn enable(
        role: &str,
        permission: &str,
        event_type: &Option<String>,
        conn: &mut Conn,
    ) -> GreaseResult<()> {
        let query = query_builder::insert(RolePermission::table_name())
            .set("role", &format!("'{}'", role))
            .set("permission", &format!("'{}'", permission))
            .set("event_type", &event_type.to_value().as_sql(false))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn disable(
        role: &str,
        permission: &str,
        event_type: &Option<String>,
        conn: &mut Conn,
    ) -> GreaseResult<()> {
        let query = query_builder::delete(RolePermission::table_name())
            .filter(&format!("role = '{}'", role))
            .filter(&format!("permission = '{}'", permission))
            .filter(&format!(
                "event_type = {}",
                event_type.to_value().as_sql(false)
            ))
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl MemberRole {
    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<(Member, Role)>> {
        let query = query_builder::select(MemberRole::table_name())
            .join(Member::table_name(), "member", "email", Join::Inner)
            .join(Role::table_name(), "role", "name", Join::Inner)
            .fields(MemberWithRoleRow::field_names())
            .build();

        crate::db::load::<MemberWithRoleRow, _>(&query, conn)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect())
    }
}

#[derive(grease_derive::FieldNames, grease_derive::FromRow)]
struct MemberWithRoleRow {
    // member fields
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
    // role fields
    pub name: String,
    pub rank: i32,
    pub max_quantity: i32,
}

impl Into<(Member, Role)> for MemberWithRoleRow {
    fn into(self) -> (Member, Role) {
        (
            Member {
                email: self.email,
                first_name: self.first_name,
                preferred_name: self.preferred_name,
                last_name: self.last_name,
                pass_hash: self.pass_hash,
                phone_number: self.phone_number,
                picture: self.picture,
                passengers: self.passengers,
                location: self.location,
                about: self.about,
                major: self.major,
                minor: self.minor,
                hometown: self.hometown,
                arrived_at_tech: self.arrived_at_tech,
                gateway_drug: self.gateway_drug,
                conflicts: self.conflicts,
                dietary_restrictions: self.dietary_restrictions,
            },
            Role {
                name: self.name,
                rank: self.rank,
                max_quantity: self.max_quantity,
            },
        )
    }
}
