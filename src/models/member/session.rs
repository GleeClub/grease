use async_graphql::Result;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::models::member::Member;
use crate::util::current_time;

pub struct Session {
    pub member: String,
    pub key: String,
}

impl Session {
    pub async fn with_token(token: &str, pool: &PgPool) -> Result<Self> {
        Self::with_token_opt(token, pool)
            .await?
            .ok_or("No login tied to the provided API token")
            .map_err(Into::into)
    }

    pub async fn with_token_opt(token: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM session WHERE key = $1", token)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    /// Determines if a password reset request has expired.
    ///
    /// Assumes the current session is a password request session, so non-password resets
    /// will return true as if they are expired sessions.
    fn is_expired_password_reset(&self) -> bool {
        let timestamp_requested = self
            .key
            .split('X')
            .nth(1)
            .and_then(|ts| ts.parse::<i64>().ok());

        if let Some(timestamp) = timestamp_requested {
            if let Ok(time_requested) = OffsetDateTime::from_unix_timestamp(timestamp) {
                return (current_time() - time_requested) > Duration::days(1);
            }
        }

        true
    }

    pub async fn get_or_generate_token(email: &str, pool: &PgPool) -> Result<String> {
        Member::with_email(email, pool).await?; // ensure that member exists

        let session = sqlx::query_scalar!("SELECT key FROM session WHERE member = $1", email)
            .fetch_optional(pool)
            .await?;
        if let Some(session_key) = session {
            if session_key.contains('X') {
                Self::remove(email, pool).await?;
            } else {
                return Ok(session_key);
            }
        }

        let token = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO session (member, key) VALUES ($1, $2)",
            email,
            token
        )
        .execute(pool)
        .await?;

        Ok(token)
    }

    pub async fn remove(email: &str, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM session WHERE member = $1", email)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn generate_for_forgotten_password(email: &str, pool: &PgPool) -> Result<()> {
        Member::with_email(email, pool).await?; // ensure that member exists

        sqlx::query!("DELETE FROM session WHERE member = $1", email)
            .execute(pool)
            .await?;
        let new_token = format!("{}X{}", Uuid::new_v4(), current_time().unix_timestamp());
        sqlx::query!(
            "INSERT INTO session (member, key) VALUES ($1, $2)",
            email,
            new_token
        )
        .execute(pool)
        .await?;

        // TODO: fix emails
        // emails::reset_password(email, new_token).send().await

        Ok(())
    }

    pub async fn reset_password(token: &str, pass_hash: &str, pool: &PgPool) -> Result<()> {
        let session = Self::with_token_opt(token, pool).await?.ok_or_else(|| {
            "No password reset request was found for the given token. \
                 Please request another password reset."
                .to_owned()
        })?;

        if session.is_expired_password_reset() {
            return Err(
                "Your token expired after 24 hours. Please request another password reset.".into(),
            );
        }

        Self::remove(&session.member, pool).await?;
        let hash = bcrypt::hash(pass_hash, 10)
            .map_err(|err| format!("Failed to hash password: {}", err))?;
        sqlx::query!(
            "UPDATE member SET pass_hash = $1 WHERE email = $2",
            hash,
            session.member
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
