//! Extra utilties for use elsewhere in the API.

use anyhow::Context;
use askama::Template;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::time::interval;

use crate::email::event::EventIn48HoursEmail;
use crate::models::event::Event;
use crate::models::semester::Semester;
use crate::util::current_time;

pub mod event;

pub const MEMBER_LIST_NAME: &str = "Glee Club Members";
pub const MEMBER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const OFFICER_LIST_NAME: &str = "Glee Club Officers";
pub const OFFICER_LIST_ADDRESS: &str = "gleeclub_officers@lists.gatech.edu";

pub trait Email: Template {
    fn subject(&self) -> String;
    fn address(&self) -> Mailbox;
}

pub async fn send_email(email: impl Email) -> anyhow::Result<()> {
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost();
    let message = Message::builder()
        .to(email.address())
        .from(Mailbox {
            name: Some(OFFICER_LIST_NAME.to_owned()),
            email: OFFICER_LIST_ADDRESS.parse().unwrap(),
        })
        .subject(email.subject())
        .header(ContentType::TEXT_HTML)
        .body(email.render().context("Failed to render email")?)
        .context("Failed to build email message")?;

    let response = mailer.send(message).await.context("Failed to send email")?;
    if !response.is_positive() {
        anyhow::bail!(
            "Failed to send email using STMP over localhost: {}",
            response.code()
        );
    }

    Ok(())
}

pub async fn run_email_loop(interval_seconds: u64, pool: PgPool) {
    let mut interval = interval(tokio::time::Duration::from_secs(interval_seconds));
    let mut last_run = current_time();

    loop {
        interval.tick().await;
        let now = current_time();

        // TODO: log this?
        let _ = send_emails(last_run.clone(), now.clone(), &pool);
        last_run = now;
    }
}

async fn send_emails(
    from: OffsetDateTime,
    to: OffsetDateTime,
    pool: &PgPool,
) -> anyhow::Result<()> {
    for event in events_to_notify_about(from, to, pool)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to load events to send emails about: {:?}", err))?
    {
        // TODO: just log on failure, don't return early?
        let email = EventIn48HoursEmail::for_event(&event, pool).await?;
        send_email(email).await?;
    }

    Ok(())
}

async fn events_to_notify_about(
    from: OffsetDateTime,
    to: OffsetDateTime,
    pool: &PgPool,
) -> async_graphql::Result<impl Iterator<Item = Event>> {
    let current_semester = Semester::get_current(pool).await?;
    let all_events = Event::for_semester(&current_semester.name, pool).await?;
    let two_days = Duration::days(2);

    Ok(all_events.into_iter().filter(move |event| {
        event.call_time.0 < (to + two_days) && event.call_time.0 > (from + two_days)
    }))
}
