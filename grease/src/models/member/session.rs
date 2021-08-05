use anyhow::bail;
use async_graphql::Result;
use chrono::{Duration, Local, NaiveDateTime};

use crate::db_conn::DbConn;

pub struct Session {
    pub member: String,
    pub key: String,
}

impl Session {
    pub async fn load_opt(token: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM session WHERE `key` = ?", token)
            .query_optional(&mut *conn)
            .await
            .into()
    }

    pub async fn load(token: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(token, conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No login tied to the provided API token"))
    }

    pub async fn get_or_generate_token(email: &str, conn: &DbConn) -> Result<String> {
        Member::load(email, conn).await?; // ensure that member exists

        let session = sqlx::query!("SELECT `key` FROM session WHERE member = ?", email)
            .query_optional(&mut *conn)
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
        .query(&mut *conn)
        .await?;

        Ok(token)
    }

    pub async fn remove(email: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .query(&mut *conn)
            .await
            .into()
    }

    pub async fn generate_for_forgotten_password(email: &str, conn: &DbConn) -> Result<()> {
        Member::load(email, conn).await?; // ensure that member exists

        sqlx::query!("DELETE FROM session WHERE member = ?", email)
            .query(&mut *conn)
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
        .query(&mut *conn)
        .await?;

        emails::reset_password(email, new_token).send().await
    }

    pub async fn reset_password(token: &str, pass_hash: &str) -> Result<()> {
        let session: Session = Self::load_opt(token, conn).await?.ok_or_else(|| {
            anyhow::anyhow!(
                "No password reset request was found for the given token. \
                 Please request another password reset."
            )
        })?;

        let time_requested = session
            .key
            .split('X')
            .nth(1)
            .and_then(|ts| ts.parse::<i64>().ok())
            .map(|ts| NaiveDateTime::from_timestamp(ts, 0));
        let now = Local::now().naive_local();
        let expired = time_requested
            .map(|time| (now - time) > Duration::days(1))
            .unwrap_or(true);
        if expired {
            bail!("Your token expired after 24 hours. Please request another password reset.");
        }

        Self::remove(session.member, conn).await?;
        let hash = bcrypt::hash(pass_hash, 10).context("Failed to hash password")?;
        sqlx::query!(
            "UPDATE member SET pass_hash = ? WHERE email = ?",
            hash,
            session.member
        )
        .query(&mut *conn)
        .await
        .into()
    }
}
