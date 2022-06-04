//! Extra utilties for use elsewhere in the API.

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use time::{Duration, OffsetDateTime};

use self::event::email_for_event;
use crate::db::DbConn;
use crate::models::event::Event;
use crate::models::semester::Semester;
use crate::util::{gql_err_to_anyhow, now};

pub mod event;
pub mod minutes;

pub const MEMBER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const OFFICER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const FROM_ADDRESS: &str = "Glee Club Officers";

pub struct Email<'a> {
    pub address: &'a str,
    pub subject: String,
    pub body: String,
}

impl<'a> Email<'a> {
    pub async fn send(&'a self) -> Result<()> {
        let mut mail = Command::new("mail")
            .args(&["-s", &self.subject, self.address])
            .stdin(Stdio::piped())
            .spawn()
            .context("Couldn't run `mail` to send an email")?;

        let stdin = mail
            .stdin
            .as_mut()
            .context("No stdin was available for `mail`")?;
        stdin
            .write_all(self.body.as_bytes())
            .context("Couldn't send an email with `mail`")?;

        let output = mail
            .wait_with_output()
            .context("The output of the `mail` command couldn't be retrieved")?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "`mail` failed to send an email with error code {}: {}",
                output.status.code().unwrap_or(1),
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }
}

pub async fn send_emails(conn: &DbConn) -> Result<()> {
    let since = now() - Duration::hours(1);

    for event in events_to_notify_about(conn, since)
        .await
        .map_err(gql_err_to_anyhow)?
    {
        let email = email_for_event(&event, conn)
            .await
            .map_err(gql_err_to_anyhow)?;
        email.send().await?;
    }

    Ok(())
}

async fn events_to_notify_about(
    conn: &DbConn,
    since: OffsetDateTime,
) -> async_graphql::Result<impl Iterator<Item = Event>> {
    let current_semester = Semester::get_current(&conn).await?;
    let all_events = Event::for_semester(&current_semester.name, &conn).await?;

    let two_days_from_now = now() + Duration::days(2);
    let two_days_from_last_checked = since + Duration::days(2);

    Ok(all_events.into_iter().filter(move |event| {
        event.call_time.0 < two_days_from_now && event.call_time.0 > two_days_from_last_checked
    }))
}
