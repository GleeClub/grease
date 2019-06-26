use db::*;
use error::*;
use pinto::query_builder::*;
use util::random_base64;

impl GoogleDoc {
    pub fn load<C: Connection>(doc_name: &str, conn: &mut C) -> GreaseResult<GoogleDoc> {
        conn.first(
            &Self::filter(&format!("name = '{}'", doc_name)),
            format!("No google doc named '{}'.", doc_name),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<GoogleDoc>> {
        conn.load(&Self::select_all_in_order("name", Order::Asc))
    }

    pub fn insert<C: Connection>(new_doc: &GoogleDoc, conn: &mut C) -> GreaseResult<()> {
        new_doc.insert(conn)
    }

    pub fn update<C: Connection>(
        old_name: &str,
        changed_doc: &GoogleDoc,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("name = '{}'", old_name))
                .set("name", &to_value(&changed_doc.name))
                .set("url", &to_value(&changed_doc.url)),
            format!("No google doc named '{}'.", old_name),
        )
    }

    pub fn delete<C: Connection>(name: &str, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Self::table_name()).filter(&format!("name = '{}'", name)),
            format!("No google doc named '{}'.", name),
        )
    }
}

impl Announcement {
    pub fn load<C: Connection>(announcement_id: i32, conn: &mut C) -> GreaseResult<Announcement> {
        conn.first(
            &Self::filter(&format!("id = {}", announcement_id)),
            format!("No announcement with id {}.", announcement_id),
        )
    }

    pub fn insert<C: Connection>(
        new_content: &str,
        member: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<i32> {
        conn.insert_returning_id(
            Insert::new(Self::table_name())
                .set("member", &to_value(member))
                .set("semester", &to_value(semester))
                .set("content", &to_value(new_content)),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Announcement>> {
        conn.load(&Self::select_all_in_order("time", Order::Desc))
    }

    pub fn load_all_for_semester<C: Connection>(
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Announcement>> {
        conn.load(
            Self::select_all()
                .filter(&format!("semester = '{}'", semester))
                .filter("archived = false")
                .order_by("time", Order::Desc),
        )
    }

    pub fn archive<C: Connection>(announcement_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("id = {}", announcement_id))
                .set("archived", "true"),
            format!("No announcement with id {}.", announcement_id),
        )
    }
}

impl Uniform {
    pub fn load<C: Connection>(id: i32, conn: &mut C) -> GreaseResult<Uniform> {
        conn.first(
            &Self::filter(&format!("id = {}", id)),
            format!("No uniform with id {}.", id),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Uniform>> {
        conn.load(&Self::select_all_in_order("name", Order::Asc))
    }

    pub fn update<C: Connection>(id: i32, updated: &Uniform, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("id = {}", id))
                .set("name", &to_value(&updated.name))
                .set("color", &to_value(&updated.color))
                .set("description", &to_value(&updated.description)),
            format!("No uniform with id {}.", id),
        )
    }

    pub fn delete<C: Connection>(id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Self::table_name()).filter(&format!("id = {}", id)),
            format!("No uniform with id {}.", id),
        )
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
    pub fn load<C: Connection>(type_name: &str, conn: &mut C) -> GreaseResult<MediaType> {
        conn.first(
            &Self::filter(&format!("name = '{}'", type_name)),
            format!("No media type named {}.", type_name),
        )
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<MediaType>> {
        conn.load(&Self::select_all_in_order("`order`", Order::Asc))
    }
}

impl Variable {
    pub fn load<C: Connection>(key: &str, conn: &mut C) -> GreaseResult<Option<Variable>> {
        conn.first_opt(&Self::filter(&format!("`key` = '{}'", key)))
    }

    pub fn set<C: Connection>(
        key: String,
        value: String,
        conn: &mut C,
    ) -> GreaseResult<Option<String>> {
        if let Some(variable) = Variable::load(&key, conn)? {
            conn.update_opt(
                Update::new(Self::table_name())
                    .filter(&format!("`key` = '{}'", &key))
                    .set("value", &value),
            )?;

            Ok(Some(variable.value))
        } else {
            let new_var = Variable { key, value };
            new_var.insert(conn)?;

            Ok(None)
        }
    }

    pub fn unset<C: Connection>(key: &str, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Self::table_name()).filter(&format!("`key` = '{}'", key)),
            format!("No variable with key {}.", key),
        )
    }
}

impl Session {
    pub fn load<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<Option<Session>> {
        conn.first_opt(&Self::filter(&format!("member = '{}'", email)))
    }

    pub fn delete<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<()> {
        conn.delete(
            Delete::new(Self::table_name()).filter(&format!("member = '{}'", email)),
            format!("No session for member {}.", email),
        )
    }

    pub fn generate<C: Connection>(given_email: &str, conn: &mut C) -> GreaseResult<String> {
        let new_session = Session {
            member: given_email.to_owned(),
            key: random_base64(32)?,
        };

        new_session.insert(conn).map(|_| new_session.key)
    }
}

impl GigSong {
    pub fn load_for_event<C: Connection>(
        event_id: i32,
        conn: &mut C,
    ) -> GreaseResult<Vec<GigSong>> {
        conn.load(&Self::filter(&format!("event = {}", event_id)).order_by("`order`", Order::Asc))
    }

    pub fn update_for_event(
        event_id: i32,
        updated_setlist: Vec<NewGigSong>,
        conn: &mut DbConn,
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

        conn.transaction(|transaction| {
            transaction.delete_opt(
                &Delete::new(Self::table_name()).filter(&format!("event = {}", event_id)),
            )?;
            for gig_song in &gig_songs {
                gig_song.insert(transaction)?;
            }

            Ok(())
        })
    }
}

impl Todo {
    pub fn load<C: Connection>(todo_id: i32, conn: &mut C) -> GreaseResult<Todo> {
        conn.first(
            &Self::filter(&format!("id = {}", todo_id)),
            format!("No todo with id {}.", todo_id),
        )
    }

    pub fn load_all_for_member<C: Connection>(
        member: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Todo>> {
        conn.load(&Self::filter(&format!(
            "member = '{}' AND completed = true",
            member
        )))
    }

    pub fn create(new_todo: NewTodo, conn: &mut DbConn) -> GreaseResult<()> {
        conn.transaction(|transaction| {
            for member in &new_todo.members {
                transaction.insert(
                    Insert::new(Self::table_name())
                        .set("`text`", &to_value(&new_todo.text))
                        .set("member", &to_value(&member)),
                )?;
            }

            Ok(())
        })
    }

    pub fn mark_complete<C: Connection>(todo_id: i32, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("id = {}", todo_id))
                .set("completed", "true"),
            format!("No todo with id {}.", todo_id),
        )
    }
}

impl RolePermission {
    pub fn enable<C: Connection>(
        role: &str,
        permission: &str,
        event_type: &Option<String>,
        conn: &mut C,
    ) -> GreaseResult<()> {
        if conn
            .first_opt::<RolePermission>(&Self::filter(&format!(
                "role = '{}' AND permission = '{}' AND event_type = '{}'",
                role,
                permission,
                to_value(&event_type)
            )))?
            .is_some()
        {
            Ok(())
        } else {
            conn.insert(
                Insert::new(Self::table_name())
                    .set("role", &to_value(role))
                    .set("permission", &to_value(permission))
                    .set("event_type", &to_value(event_type)),
            )
        }
    }

    pub fn disable<C: Connection>(
        role: &str,
        permission: &str,
        event_type: &Option<String>,
        conn: &mut C,
    ) -> GreaseResult<()> {
        conn.delete_opt(
            Delete::new(Self::table_name())
                .filter(&format!("role = '{}'", role))
                .filter(&format!("permission = '{}'", permission))
                .filter(&format!("event_type = {}", to_value(event_type))),
        )
    }
}

impl MemberRole {
    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<(Member, Role)>> {
        conn.load_as::<MemberWithRoleRow, _>(
            Select::new(MemberRole::table_name())
                .join(Member::table_name(), "member", "email", Join::Inner)
                .join(Role::table_name(), "role", "name", Join::Inner)
                .fields(MemberWithRoleRow::field_names()),
        )
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
