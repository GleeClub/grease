//! Extra utilties for use elsewhere in the API.

use anyhow::Context;
use askama::Template;
use mailgun_v3::email::{self, Message, MessageBody};
use mailgun_v3::{Credentials, EmailAddress};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::time::interval;

use crate::email::event::EventIn48HoursEmail;
use crate::models::event::Event;
use crate::models::semester::Semester;
use crate::util::current_time;

pub mod event;
pub mod reset_password;

pub const MEMBER_LIST_NAME: &str = "Glee Club Members";
// pub const MEMBER_LIST_ADDRESS: &str = "gleeclub@lists.gatech.edu";
pub const MEMBER_LIST_ADDRESS: &str = "sam.mohr@protonmail.com";

pub const MAILGUN_NAME: &str = "GlubHub";
pub const MAILGUN_EMAIL: &str = "mail@glubhub.org";
pub const MAILGUN_DOMAIN: &str = "mail.glubhub.org";

pub trait Email: Template {
    fn subject(&self) -> String;
    fn address(&self) -> EmailAddress;
}

pub async fn send_email(email: impl Email) -> anyhow::Result<()> {
    let token = std::env::var("MAILGUN_TOKEN").context("`MAILGUN_TOKEN` not set")?;
    let creds = Credentials::new(token, MAILGUN_DOMAIN);

    let sender =
        EmailAddress::name_address(MAILGUN_NAME.to_owned(), MAILGUN_EMAIL.parse().unwrap());
    let message = Message {
        to: vec![email.address()],
        subject: email.subject(),
        body: MessageBody::Html(email.render().context("Failed to render email")?),
        ..Default::default()
    };

    email::async_impl::send_email(&creds, &sender, message)
        .await
        .map(|_| ())
        .map_err(|err| anyhow::anyhow!("Failed to send email: {err}"))
}

pub async fn run_email_loop(interval_seconds: u64, pool: PgPool) {
    let mut interval = interval(tokio::time::Duration::from_secs(interval_seconds));
    let mut last_run = current_time();

    loop {
        interval.tick().await;
        let now = current_time();

        println!(
            "Sending emails for events from {:?} to {:?}",
            last_run + Duration::days(2),
            now + Duration::days(2)
        );
        send_emails(last_run.clone(), now.clone(), &pool).await;
        last_run = now;
    }
}

async fn send_emails(from: OffsetDateTime, to: OffsetDateTime, pool: &PgPool) {
    let events = match events_to_notify_about(from, to, pool).await {
        Ok(events) => events,
        Err(error) => {
            eprintln!(
                "Failed to load events to send emails about: {:?}",
                error.message
            );
            return;
        }
    };
    println!(
        "Found {} events: {}",
        events.len(),
        events
            .iter()
            .map(|event| format!("`{}`", event.name))
            .collect::<Vec<_>>()
            .join(", ")
    );

    for event in events {
        match EventIn48HoursEmail::for_event(&event, pool).await {
            Err(error) => {
                eprintln!(
                    "Failed to create email content for upcoming event `{}`: {:?}",
                    event.name, error
                );
            }
            Ok(email) => {
                if let Err(error) = send_email(email).await {
                    eprintln!(
                        "Failed to send email for upcoming event `{}`: {:?}",
                        event.name, error
                    );
                }
            }
        }
    }
}

async fn events_to_notify_about(
    from: OffsetDateTime,
    to: OffsetDateTime,
    pool: &PgPool,
) -> async_graphql::Result<Vec<Event>> {
    let current_semester = Semester::get_current(pool).await?;
    let all_events = Event::for_semester(&current_semester.name, pool).await?;
    let two_days = Duration::days(2);

    Ok(all_events
        .into_iter()
        .filter(move |event| {
            event.call_time < (to + two_days) && event.call_time >= (from + two_days)
        })
        .collect())
}
