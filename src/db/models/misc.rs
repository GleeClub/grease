use db::{
    Announcement, GigSong, GoogleDoc, MediaType, Member, MemberRole, NewGigSong, NewTodo,
    NewUniform, Role, RolePermission, Session, Song, Todo, Uniform, Variable,
};
use diesel::{Connection, MysqlConnection};
use error::*;
use uuid::Uuid;

impl GoogleDoc {
    pub fn load<C: Connection>(doc_name: &str, conn: &mut C) -> GreaseResult<GoogleDoc> {
        use db::schema::google_docs::dsl::*;

        google_docs
            .filter(name.eq(doc_name))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<GoogleDoc>> {
        use db::schema::google_docs::dsl::*;

        google_docs
            .order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn insert<C: Connection>(new_doc: &GoogleDoc, conn: &mut C) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::insert_into(google_docs)
            .values(new_doc)
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update<C: Connection>(
        old_name: &str,
        changed_doc: &GoogleDoc,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::update(google_docs.filter(name.eq(old_name)))
            .set(changed_doc)
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn delete<C: Connection>(given_name: &str, conn: &mut C) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::delete(google_docs.filter(name.eq(given_name)))
            .execute(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Announcement {
    pub fn load<C: Connection>(announcement_id: i32, conn: &mut C) -> GreaseResult<Announcement> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(id.eq(announcement_id))
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn insert<C: Connection>(
        new_content: &str,
        given_member: &str,
        given_semester: &str,
        conn: &mut C,
    ) -> GreaseResult<i32> {
        use db::schema::announcement::dsl::*;

        conn.transaction(|| {
            diesel::insert_into(announcement)
                .values((
                    member.eq(given_member),
                    semester.eq(given_semester),
                    content.eq(new_content),
                ))
                .execute(conn)?;

            announcement.select(id).order_by(id.desc()).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .order_by(time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_semester<C: Connection>(
        given_semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(semester.eq(given_semester).and(archived.eq(false)))
            .order_by(time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn archive<C: Connection>(announcement_id: i32, conn: &mut C) -> GreaseResult<()> {
        use db::schema::announcement::dsl::*;

        diesel::update(announcement.filter(id.eq(announcement_id)))
            .set(archived.eq(true))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No announcement with id {}.", announcement_id),
    }
}

impl Uniform {
    pub fn load<C: Connection>(uniform_id: i32, conn: &mut C) -> GreaseResult<Uniform> {
        use db::schema::uniform::dsl::*;

        uniform
            .filter(id.eq(uniform_id))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No uniform with id {}.", id),
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Uniform>> {
        use db::schema::uniform::dsl::*;

        uniform
            .order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update<C: Connection>(
        uniform_id: i32,
        updated: &NewUniform,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::uniform::dsl::*;

        diesel::update(uniform.filter(id.eq(uniform_id)))
            .values(updated)
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No uniform with id {}.", id),
    }

    pub fn delete<C: Connection>(uniform_id: i32, conn: &mut C) -> GreaseResult<()> {
        use db::schema::uniform::dsl::*;

        diesel::delete(uniform.filter(id.eq(uniform_id)))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No uniform with id {}.", id),
    }

    pub fn validate_color(color: &Option<String>) -> GreaseResult<()> {
        let regex = regex::Regex::new(r"^#(\w{3}|\w{6})$").unwrap();

        // if color string is invalid
        if color
            .as_ref()
            .map(|color| !regex.is_match(&color))
            .unwrap_or(false)
        {
            Err(GreaseError::BadRequest(
                "uniform colors must be in the format '#XXX' or '#XXXXXX', where X is a hexadecimal number"
                    .to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

impl MediaType {
    pub fn load<C: Connection>(type_name: &str, conn: &mut C) -> GreaseResult<MediaType> {
        use db::schema::media_type::dsl::*;

        media_type
            .filter(name.eq(type_name))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No media type named {}.", type_name),
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<MediaType>> {
        use db::schema::media_type::dsl::*;

        media_type
            .order_by(order.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Variable {
    pub fn load<C: Connection>(given_key: &str, conn: &mut C) -> GreaseResult<Option<Variable>> {
        use db::schema::variable::dsl::*;

        variable
            .filter(key.eq(given_key))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn set<C: Connection>(
        given_key: String,
        given_value: String,
        conn: &mut C,
    ) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        if let Some(var) = Variable::load(&given_key, conn)? {
            diesel::update(variable.filter(key.eq(given_key)))
                .set(value.eq(given_value))
                .execute(conn)
                .map(|_| Some(var.value))
                .map_err(GreaseError::DbError)
        } else {
            diesel::insert_into(variable)
                .values((key.eq(given_key), value.eq(given_value)))
                .execute(conn)
                .map(|_| None)
                .map_err(GreaseError::DbError)
        }
    }

    pub fn unset<C: Connection>(given_key: &str, conn: &mut C) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        let old_val = Variable::load(given_key, conn)?.map(|var| var.value);
        diesel::delete(variable.filter(key.eq(given_key)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(old_val)
    }
}

impl Session {
    pub fn load<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<Option<Session>> {
        use db::schema::session::dsl::*;

        session
            .filter(member.eq(email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn delete<C: Connection>(email: &str, conn: &mut C) -> GreaseResult<()> {
        use db::schema::session::dsl::*;

        diesel::delete(session.filter(member.eq(email)))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No session for member {}.", email),
    }

    pub fn generate<C: Connection>(given_email: &str, conn: &mut C) -> GreaseResult<String> {
        use db::schema::session::dsl::*;

        let new_key = Uuid::new_v4().to_string();
        diesel::insert_into(session)
            .values((member.eq(given_email), key.eq(new_key)))
            .execute(conn)
            .map(|_| new_key)
            .map_err(GreaseError::DbError)
    }
}

impl GigSong {
    pub fn load_for_event<C: Connection>(event_id: i32, conn: &mut C) -> GreaseResult<Vec<Song>> {
        use db::schema::gig_song::dsl::*;

        gig_song
            .inner_join(crate::db::schema::song::table)
            .select(gig_song)
            .filter(event.eq(event_id))
            .order_by(order.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update_for_event(
        event_id: i32,
        updated_setlist: Vec<NewGigSong>,
        conn: &mut MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::gig_song::dsl::*;

        let gig_songs = updated_setlist
            .into_iter()
            .enumerate()
            .map(|(index, gig_song)| GigSong {
                event: event_id,
                song: gig_song.song,
                order: index as i32 + 1,
            })
            .collect::<Vec<GigSong>>();

        conn.transaction(|| {
            diesel::delete(gig_song.filter(event.eq(event_id))).execute(conn)?;
            diesel::insert_into(gig_song)
                .values(&gig_songs)
                .execute(conn)?;

            Ok(())
        })
    }
}

impl Todo {
    pub fn load<C: Connection>(todo_id: i32, conn: &mut C) -> GreaseResult<Todo> {
        use db::schema::todo::dsl::*;

        todo.filter(id.eq(todo_id))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No todo with id {}.", todo_id),
    }

    pub fn load_all_for_member<C: Connection>(
        given_member: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Todo>> {
        use db::schema::todo::dsl::*;

        todo.filter(member.eq(given_member).and(completed.eq(true)))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create<C: Connection>(new_todo: NewTodo, conn: &mut C) -> GreaseResult<()> {
        use db::schema::todo::dsl::*;

        diesel::insert_into(todo)
            .values(&new_todo)
            .execute(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn mark_complete<C: Connection>(todo_id: i32, conn: &mut C) -> GreaseResult<()> {
        use db::schema::todo::dsl::*;

        diesel::update(todo.filter(id.eq(todo_id)))
            .set(completed.eq(true))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No todo with id {}.", todo_id),
    }
}

impl RolePermission {
    pub fn enable<C: Connection>(
        given_role: &str,
        given_permission: &str,
        given_event_type: &Option<String>,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::role_permission::dsl::*;

        let already_exists = role_permission
            .filter(
                role.eq(given_role).and(
                    permission
                        .eq(given_permission)
                        .and(event_type.eq(given_event_type)),
                ),
            )
            .first(conn)
            .optional()
            .map_err(GreaseResult::DbError)?
            .is_some();

        if already_exists {
            Ok(())
        } else {
            diesel::insert_into(role_permission)
                .values((
                    role.eq(given_role),
                    permission.eq(given_permission),
                    event_type.eq(given_event_type),
                ))
                .execute(conn)
                .map_err(GreaseError::DbError)
        }
    }

    pub fn disable<C: Connection>(
        given_role: &str,
        given_permission: &str,
        given_event_type: &Option<String>,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::role_permission::dsl::*;

        diesel::delete(
            role_permission.filter(
                role.eq(given_role)
                    .and(permission.eq(given_permission))
                    .and(event_type.eq(given_event_type)),
            ),
        )
        .execute(conn)
        .map_err(GreaseError::DbError)
    }
}

// TODO: figure out what max quantity actually entails
impl MemberRole {
    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<(Member, Role)>> {
        use db::schema::{member, member_role::dsl::*, role};

        member_role
            .inner_join(member::table)
            .inner_join(role::table)
            .order_by(role::dsl::rank.asc())
            .load::<Vec<Member, Role>>(conn)
            .map_err(GreaseError::DbError)
    }
}
