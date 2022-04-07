use async_graphql::Result;
use uuid::Uuid;
use crate::db_conn::DbConn;
use crate::models::member::member::Member;
use chrono::{Duration, Local, NaiveDateTime};

use crate::db_conn::DbConn;

pub struct Session {
    pub member: String,
    pub key: String,
}

impl Session {
    pub async fn with_token(token: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(token, conn)
            .await?
            .ok_or_else(|| "No login tied to the provided API token".to_owned())
    }

    pub async fn with_token_opt(token: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM session WHERE `key` = ?", token)
            .query_optional(&conn)
            .await
    }

    /// Determines if a password reset request has expired.
    ///
    /// Assumes the current session is a password request session, so non-password resets
    /// will return true as if they are expired sessions.
    fn is_expired_password_reset(&self) -> bool {
        let time_requested = session
            .key
            .split('X')
            .nth(1)
            .and_then(|ts| ts.parse::<i64>().ok())
            .map(|ts| NaiveDateTime::from_timestamp(ts, 0));

        if let Some(time_requested) = time_requested {
            let now = Local::now().naive_local();
            (now - time_requested) > Duration::days(1)
        } else {
            true
        }
    }

    pub async fn get_or_generate_token(email: &str, conn: &DbConn) -> Result<String> {
        Member::load(email, conn).await?; // ensure that member exists

        let session = sqlx::query!("SELECT `key` FROM session WHERE member = ?", email)
            .query_optional(&conn)
            .await?;
        if let Some(session_key) = session {
            return Ok(session_key);
        }

        let token = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO session (member, `key`) VALUES (?, ?)",
            email,
            token
        )
        .query(&conn)
        .await?;

        Ok(token)
    }

    pub async fn remove(email: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .query(&conn)
            .await
            .into()
    }

    pub async fn generate_for_forgotten_password(email: &str, conn: &DbConn) -> Result<()> {
        Member::load(email, conn).await?; // ensure that member exists

        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .query(&conn)
            .await?;
        let new_token = format!(
            "{}X{}",
            Uuid::new_v4().to_string()[..32],
            Local::now().timestamp()
        );
        sqlx::query!(
            "INSERT INTO session (member, `key`) VALUES (?, ?)",
            email,
            new_token
        )
        .query(&conn)
        .await?;

        emails::reset_password(email, new_token).send().await
    }

    pub async fn reset_password(token: &str, pass_hash: &str,conn: &DbConn) -> Result<()> {
        let session = Self::with_token_opt(token, conn).await?.ok_or_else(|| {
                "No password reset request was found for the given token. \
                 Please request another password reset.".to_owned()
        })?;

        if session.is_expired_password_reset() {
            return Err("Your token expired after 24 hours. Please request another password reset.".to_owned());
        }

        Self::remove(session.member, conn).await?;
        let hash = bcrypt::hash(pass_hash, 10).context("Failed to hash password")?;
        sqlx::query!(
            "UPDATE member SET pass_hash = ? WHERE email = ?",
            hash,
            session.member
        )
        .query(&conn)
        .await
    }
}
