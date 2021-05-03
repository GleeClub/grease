use chrono::{Duration, NaiveDateTime, Utc};
use db::{
    Announcement, GigSong, GoogleDoc, MediaType, Member, MemberRole, NewGigSong, NewTodo,
    NewUniform, PasswordReset, Role, RolePermission, Session, Song, Todo, Uniform, Variable,
};
use diesel::prelude::*;
use error::*;
use uuid::Uuid;

impl GoogleDoc {
    pub fn load(doc_name: &str, conn: &MysqlConnection) -> GreaseResult<GoogleDoc> {
        use db::schema::google_docs::dsl::*;

        google_docs
            .filter(name.eq(doc_name))
            .first(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "No google docs found with the name {}.",
                doc_name
            )))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<GoogleDoc>> {
        use db::schema::google_docs::dsl::*;

        google_docs
            .order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn insert(new_doc: &GoogleDoc, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::insert_into(google_docs)
            .values(new_doc)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn update(
        old_name: &str,
        changed_doc: &GoogleDoc,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::update(google_docs.filter(name.eq(old_name)))
            .set(changed_doc)
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }

    pub fn delete(given_name: &str, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::google_docs::dsl::*;

        diesel::delete(google_docs.filter(name.eq(given_name)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
    }
}

impl Announcement {
    pub fn load(announcement_id: i32, conn: &MysqlConnection) -> GreaseResult<Announcement> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(id.eq(announcement_id))
            .first(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn insert(
        new_content: &str,
        given_member: &str,
        given_semester: &str,
        conn: &MysqlConnection,
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

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .order_by(time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_semester(
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Announcement>> {
        use db::schema::announcement::dsl::*;

        announcement
            .filter(semester.eq(given_semester).and(archived.eq(false)))
            .order_by(time.desc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn archive(announcement_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::announcement::dsl::*;

        diesel::update(announcement.filter(id.eq(announcement_id)))
            .set(archived.eq(true))
            .execute(conn)?;

        Ok(())
        // format!("No announcement with id {}.", announcement_id),
    }
}

impl Uniform {
    pub fn load(uniform_id: i32, conn: &MysqlConnection) -> GreaseResult<Uniform> {
        use db::schema::uniform::dsl::*;

        uniform
            .filter(id.eq(uniform_id))
            .first(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "No uniform with id {}.",
                uniform_id
            )))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Uniform>> {
        use db::schema::uniform::dsl::*;

        uniform
            .order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_default(conn: &MysqlConnection) -> GreaseResult<Uniform> {
        use db::schema::uniform::dsl::*;

        uniform
            .order_by(name.asc())
            .first(conn)
            .optional()?
            .ok_or(GreaseError::ServerError(
                "There are currently no uniforms.".to_owned(),
            ))
    }

    pub fn update(
        uniform_id: i32,
        updated: &NewUniform,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::uniform::dsl::*;

        diesel::update(uniform.filter(id.eq(uniform_id)))
            .set(updated)
            .execute(conn)?;

        Ok(())
        // format!("No uniform with id {}.", id),
    }

    pub fn delete(uniform_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::uniform::dsl::*;

        diesel::delete(uniform.filter(id.eq(uniform_id))).execute(conn)?;

        Ok(())
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
    pub fn load(type_name: &str, conn: &MysqlConnection) -> GreaseResult<MediaType> {
        use db::schema::media_type::dsl::*;

        media_type
            .filter(name.eq(type_name))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No media type named {}.", type_name),
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<MediaType>> {
        use db::schema::media_type::dsl::*;

        media_type
            .order_by(order.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Variable {
    pub fn load(given_key: &str, conn: &MysqlConnection) -> GreaseResult<Option<Variable>> {
        use db::schema::variable::dsl::*;

        variable
            .filter(key.eq(given_key))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn set(
        given_key: String,
        given_value: String,
        conn: &MysqlConnection,
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

    pub fn unset(given_key: &str, conn: &MysqlConnection) -> GreaseResult<Option<String>> {
        use db::schema::variable::dsl::*;

        let old_val = Variable::load(given_key, conn)?.map(|var| var.value);
        diesel::delete(variable.filter(key.eq(given_key)))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(old_val)
    }
}

impl Session {
    pub fn load_for_email(email: &str, conn: &MysqlConnection) -> GreaseResult<Option<Session>> {
        use db::schema::session::dsl::*;

        session
            .filter(member.eq(email))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn load_for_token(token: &str, conn: &MysqlConnection) -> GreaseResult<Option<Session>> {
        use db::schema::session::dsl::*;

        session
            .filter(key.eq(token))
            .first(conn)
            .optional()
            .map_err(GreaseError::DbError)
    }

    pub fn delete(email: &str, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::session::dsl::*;

        diesel::delete(session.filter(member.eq(email))).execute(conn)?;

        Ok(())
        // format!("No session for member {}.", email),
    }

    pub fn generate(given_email: &str, conn: &MysqlConnection) -> GreaseResult<String> {
        use db::schema::session::dsl::*;

        let new_key = Uuid::new_v4().to_string();
        diesel::insert_into(session)
            .values((member.eq(given_email), key.eq(&new_key)))
            .execute(conn)
            .map(|_| new_key)
            .map_err(GreaseError::DbError)
    }

    pub fn generate_for_forgotten_password(
        email: String,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::session::dsl::*;
        use util::Email;

        let _given_member = Member::load(&email, &conn)?;

        conn.transaction(|| {
            diesel::delete(session.filter(member.eq(&email))).execute(conn)?;
            let now = chrono::Utc::now().timestamp_millis();
            let rand_string = Uuid::new_v4()
                .to_string()
                .chars()
                .take(32)
                .collect::<String>();
            let new_token = format!("{}X{}", rand_string, now);

            diesel::insert_into(session)
                .values((member.eq(&email), key.eq(&new_token)))
                .execute(conn)?;

            let reset_url = format!(
                "https://gleeclub.gatech.edu/glubhub/#/reset-password/{}",
                new_token
            );

            Email {
                to_address: email,
                subject: "Reset Your Password".to_owned(),
                content: "".to_owned(),
                // content: html! {
                //     <p>
                //         "You have requested a password reset on your Glee Club account. \
                //         Please click "
                //         <a href=&reset_url>
                //             "here"
                //         </a>
                //         " to reset your password."
                //     </p>
                // },
            }
            .send()?;

            Ok(())
        })
    }

    pub fn reset_password(
        token: String,
        password_reset: PasswordReset,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::{member as member_table, session::dsl::*};

        let member_session =
            Session::load_for_token(&token, conn)?.ok_or(GreaseError::BadRequest(
                "No password reset request was found for the given token. \
                 Please request another password reset."
                    .to_owned(),
            ))?;

        let time_requested = member_session
            .key
            .split('X')
            .nth(1)
            .and_then(|timestamp_str| timestamp_str.parse::<i64>().ok())
            .map(|timestamp| NaiveDateTime::from_timestamp(timestamp / 1000, 0));

        if time_requested
            .map(|time| time + Duration::days(1) >= Utc::now().naive_utc())
            .unwrap_or(false)
        {
            let pass_hash = bcrypt::hash(&password_reset.pass_hash, 10).map_err(|err| {
                GreaseError::ServerError(format!("Unable to hash new password: {}", err))
            })?;
            conn.transaction(|| {
                diesel::delete(session.filter(member.eq(&member_session.member))).execute(conn)?;

                diesel::update(
                    member_table::table.filter(member_table::email.eq(&member_session.member)),
                )
                .set(member_table::pass_hash.eq(pass_hash))
                .execute(conn)?;

                Ok(())
            })
        } else {
            Err(GreaseError::BadRequest(
                "Your token expired after 24 hours. Please request another password reset."
                    .to_owned(),
            ))
        }
    }
}

impl GigSong {
    pub fn load_for_event(event_id: i32, conn: &MysqlConnection) -> GreaseResult<Vec<Song>> {
        use db::schema::gig_song::dsl::*;
        use db::schema::song;

        gig_song
            .inner_join(song::table)
            .filter(event.eq(event_id))
            .order_by(order.asc())
            .load::<(GigSong, Song)>(conn)
            .map(|rows| rows.into_iter().map(|(_, s)| s).collect())
            .map_err(GreaseError::DbError)
    }

    pub fn update_for_event(
        event_id: i32,
        updated_setlist: Vec<NewGigSong>,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::gig_song::dsl::*;

        let gig_songs = updated_setlist
            .into_iter()
            .enumerate()
            .map(|(index, given_gig_song)| GigSong {
                event: event_id,
                song: given_gig_song.song,
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
    pub fn load(todo_id: i32, conn: &MysqlConnection) -> GreaseResult<Todo> {
        use db::schema::todo::dsl::*;

        todo.filter(id.eq(todo_id))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No todo with id {}.", todo_id),
    }

    pub fn load_all_for_member(
        given_member: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Todo>> {
        use db::schema::todo::dsl::*;

        todo.filter(member.eq(given_member).and(completed.eq(true)))
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn create(new_todo: NewTodo, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::todo::dsl::*;

        let todo_text = new_todo.text;
        let new_todos = new_todo
            .members
            .into_iter()
            .map(|todo_member| (text.eq(todo_text.clone()), member.eq(todo_member)))
            .collect::<Vec<_>>();
        diesel::insert_into(todo).values(&new_todos).execute(conn)?;

        Ok(())
    }

    pub fn mark_complete(todo_id: i32, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::todo::dsl::*;

        diesel::update(todo.filter(id.eq(todo_id)))
            .set(completed.eq(true))
            .execute(conn)?;

        Ok(())
        // format!("No todo with id {}.", todo_id),
    }
}

impl RolePermission {
    pub fn enable(
        given_role: &str,
        given_permission: &str,
        given_event_type: &Option<String>,
        conn: &MysqlConnection,
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
            .first::<RolePermission>(conn)
            .optional()?
            .is_some();

        if !already_exists {
            diesel::insert_into(role_permission)
                .values((
                    role.eq(given_role),
                    permission.eq(given_permission),
                    event_type.eq(given_event_type),
                ))
                .execute(conn)?;
        }

        Ok(())
    }

    pub fn disable(
        given_role: &str,
        given_permission: &str,
        given_event_type: &Option<String>,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::role_permission::dsl::*;

        diesel::delete(
            role_permission.filter(
                role.eq(given_role)
                    .and(permission.eq(given_permission))
                    .and(event_type.eq(given_event_type)),
            ),
        )
        .execute(conn)?;

        Ok(())
    }
}

// TODO: figure out what max quantity actually entails
impl MemberRole {
    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<(Member, Role)>> {
        use db::schema::{member, member_role::dsl::*, role};

        member_role
            .inner_join(member::table)
            .inner_join(role::table)
            .order_by(role::rank.asc())
            .load::<(MemberRole, Member, Role)>(conn)
            .map(|rows| rows.into_iter().map(|(_, m, r)| (m, r)).collect())
            .map_err(GreaseError::DbError)
    }
}
