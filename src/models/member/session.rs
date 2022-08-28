use async_graphql::Result;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::email::reset_password::ResetPasswordEmail;
use crate::email::send_email;
use crate::models::member::Member;
use crate::util::current_time;

/// A login session for a member
pub struct Session {
    /// The email of the member
    pub member: String,
    /// A random session key
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
        sqlx::query_as!(Self, "SELECT * FROM sessions WHERE key = $1", token)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get_or_generate_token(email: &str, pool: &PgPool) -> Result<String> {
        Member::with_email(email, pool).await?; // ensure that member exists

        let session = sqlx::query_scalar!("SELECT key FROM sessions WHERE member = $1", email)
            .fetch_optional(pool)
            .await?;
        if let Some(session_key) = session {
            return Ok(session_key);
        }

        let token = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO sessions (member, key) VALUES ($1, $2)",
            email,
            token
        )
        .execute(pool)
        .await?;

        Ok(token)
    }

    pub async fn remove(email: &str, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE member = $1", email)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// A 24-hour password reset for a member
pub struct PasswordReset {
    /// The email of the member
    pub member: String,
    /// When the password reset was requested
    pub time: OffsetDateTime,
    /// The random token emailed to the member
    pub token: String,
}

impl PasswordReset {
    pub async fn generate(email: &str, pool: &PgPool) -> Result<()> {
        let member = Member::with_email(email, pool).await?;

        let new_token = Uuid::new_v4().to_string();
        sqlx::query!("DELETE FROM password_resets WHERE member = $1", email)
            .execute(pool)
            .await?;
        sqlx::query!(
            "INSERT INTO password_resets (member, token) VALUES ($1, $2)",
            email,
            new_token
        )
        .execute(pool)
        .await?;

        let email = ResetPasswordEmail {
            member: &member,
            token: &new_token,
        };
        send_email(email).await?;

        Ok(())
    }

    pub async fn reset_from_token(token: &str, pass_hash: &str, pool: &PgPool) -> Result<()> {
        let session = sqlx::query_as!(
            PasswordReset,
            "SELECT * FROM password_resets WHERE token = $1",
            token
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            "No password reset request was found for the given token, please request another"
        })?;

        if session.time > current_time() + Duration::DAY {
            return Err("Your token expired after 24 hours, please request another reset".into());
        }

        Self::remove(&session.member, pool).await?;
        let hash = bcrypt::hash(pass_hash, 10)
            .map_err(|err| format!("Failed to hash password: {}", err))?;
        sqlx::query!(
            "UPDATE members SET pass_hash = $1 WHERE email = $2",
            hash,
            session.member
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(email: &str, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM password_resets WHERE member = $1", email)
            .execute(pool)
            .await?;

        Ok(())
    }
}
