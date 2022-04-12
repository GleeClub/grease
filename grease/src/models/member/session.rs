use async_graphql::Result;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::db::DbConn;
use crate::models::member::Member;

pub struct Session {
    pub member: String,
    pub key: String,
}

impl Session {
    pub async fn with_token(token: &str, conn: DbConn<'_>) -> Result<Self> {
        Self::with_token_opt(token, conn)
            .await?
            .ok_or_else(|| "No login tied to the provided API token".to_owned())
            .into()
    }

    pub async fn with_token_opt(token: &str, conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM session WHERE `key` = ?", token)
            .fetch_optional(&conn)
            .await
            .into()
    }

    /// Determines if a password reset request has expired.
    ///
    /// Assumes the current session is a password request session, so non-password resets
    /// will return true as if they are expired sessions.
    fn is_expired_password_reset(&self) -> Result<bool> {
        let timestamp_requested = self
            .key
            .split('X')
            .nth(1)
            .and_then(|ts| ts.parse::<i32>().ok());

        if let Some(timestamp) = timestamp_requested {
            let time_requested = OffsetDateTime::from_unix_timestamp(timestamp)
                .context("Failed to parse timestamp")?;
            let now = OffsetDateTime::now_local().context("Failed to get current time")?;
            Ok((now - time_requested) > Duration::days(1))
        } else {
            Ok(true)
        }
    }

    pub async fn get_or_generate_token(email: &str, conn: DbConn<'_>) -> Result<String> {
        Member::load(email, conn).await?; // ensure that member exists

        let session = sqlx::query!("SELECT `key` FROM session WHERE member = ?", email)
            .fetch_optional(&conn)
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
        .execute(&conn)
        .await?;

        Ok(token)
    }

    pub async fn remove(email: &str, conn: DbConn<'_>) -> Result<()> {
        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .execute(&conn)
            .await
            .into()
    }

    pub async fn generate_for_forgotten_password(email: &str, conn: DbConn<'_>) -> Result<()> {
        Member::with_email(email, conn).await?; // ensure that member exists

        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .execute(conn)
            .await?;
        let new_token = format!(
            "{}X{}",
            Uuid::new_v4().to_string()[..32],
            OffsetDateTime::now_local().timestamp()
        );
        sqlx::query!(
            "INSERT INTO session (member, `key`) VALUES (?, ?)",
            email,
            new_token
        )
        .execute(conn)
        .await?;

        // TODO: fix emails
        // emails::reset_password(email, new_token).send().await

        Ok(())
    }

    pub async fn reset_password(token: &str, pass_hash: &str, conn: DbConn<'_>) -> Result<()> {
        let session = Self::with_token_opt(token, conn).await?.ok_or_else(|| {
            "No password reset request was found for the given token. \
                 Please request another password reset."
                .to_owned()
        })?;

        if session.is_expired_password_reset() {
            return Err(
                "Your token expired after 24 hours. Please request another password reset."
                    .to_owned(),
            );
        }

        Self::remove(&session.member, conn).await?;
        let hash = bcrypt::hash(pass_hash, 10).context("Failed to hash password")?;
        sqlx::query!(
            "UPDATE member SET pass_hash = ? WHERE email = ?",
            hash,
            session.member
        )
        .execute(conn)
        .await
    }
}
