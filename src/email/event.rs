use askama::Template;
use mailgun_v3::email::EmailAddress;
use sqlx::PgPool;
use time::macros::format_description;
use time::OffsetDateTime;

use crate::email::{Email, MEMBER_LIST_ADDRESS, MEMBER_LIST_NAME};
use crate::models::event::Event;
use crate::util::local_offset;

#[derive(Template)]
#[template(path = "event-in-48-hours.html")]
pub struct EventIn48HoursEmail<'a> {
    event: &'a Event,
    uniform_name: Option<String>,
    start_time: String,
    end_time: Option<String>,
}

impl<'a> EventIn48HoursEmail<'a> {
    pub async fn for_event(
        event: &'a Event,
        pool: &PgPool,
    ) -> anyhow::Result<EventIn48HoursEmail<'a>> {
        let uniform_name: Option<String> = sqlx::query_scalar!(
            "SELECT uniforms.name FROM uniforms
             INNER JOIN gigs ON gigs.uniform = uniforms.id
             INNER JOIN events ON gigs.event = events.id
             WHERE events.id = $1",
            event.id
        )
        .fetch_optional(pool)
        .await?;

        Ok(Self {
            event,
            uniform_name,
            start_time: format_event_time(&event.call_time.to_offset(local_offset())),
            end_time: event
                .release_time
                .as_ref()
                .map(|t| format_event_time(&t.to_offset(local_offset()))),
        })
    }
}

impl<'a> Email for EventIn48HoursEmail<'a> {
    fn subject(&self) -> String {
        format!("{} is in 48 Hours", self.event.name)
    }

    fn address(&self) -> EmailAddress {
        EmailAddress::name_address(
            MEMBER_LIST_NAME.to_owned(),
            MEMBER_LIST_ADDRESS.parse().unwrap(),
        )
    }
}

#[derive(Template)]
#[template(path = "new-event.html")]
pub struct NewEventEmail<'a> {
    event: &'a Event,
    uniform_name: Option<String>,
    start_time: String,
    end_time: Option<String>,
}

impl<'a> NewEventEmail<'a> {
    pub async fn for_event(event: &'a Event, pool: &PgPool) -> anyhow::Result<NewEventEmail<'a>> {
        let uniform_name: Option<String> = sqlx::query_scalar!(
            "SELECT uniforms.name FROM uniforms
             INNER JOIN gigs ON gigs.uniform = uniforms.id
             INNER JOIN events ON gigs.event = events.id
             WHERE events.id = $1",
            event.id
        )
        .fetch_optional(pool)
        .await?;

        Ok(Self {
            event,
            uniform_name,
            start_time: format_event_time(&event.call_time.to_offset(local_offset())),
            end_time: event
                .release_time
                .as_ref()
                .map(|t| format_event_time(&t.to_offset(local_offset()))),
        })
    }
}

impl<'a> Email for NewEventEmail<'a> {
    fn subject(&self) -> String {
        format!("New Glee Club Event - {}", self.event.name)
    }

    fn address(&self) -> EmailAddress {
        EmailAddress::name_address(
            MEMBER_LIST_NAME.to_owned(),
            MEMBER_LIST_ADDRESS.parse().unwrap(),
        )
    }
}

fn format_event_time(event_time: &OffsetDateTime) -> String {
    let time_format = format_description!(
        "[weekday repr:short], [month repr:short] [day] \
         [hour]:[minute padding:zero] [period case:upper]"
    );

    event_time.format(time_format).unwrap()
}

#[cfg(test)]
mod tests {
    use askama::Template;

    use super::EventIn48HoursEmail;
    use crate::tests::mock::mock_event;

    #[test]
    fn event_in_48_hours_email_content_correct() {
        let event = mock_event();
        let email = EventIn48HoursEmail {
            event: &event,
            uniform_name: Some("Black Slacks".to_owned()),
            start_time: "Jan 1st, 2000 at 8:00 PM".to_owned(),
            end_time: Some("Jan 2nd, 2000 at 12:00 AM".to_owned()),
        };

        assert_eq!(
            email.render().unwrap(),
            "\
<html>
  <head></head>
  <body>
    <h2>
      <a href=\"https://glubhub.org/#/events/1\">
        Mock Event
      </a>
    </h2>

    <p>
      <b>Tutti Gig</b>
      
          from
          <b>Jan 1st, 2000 at 8:00 PM</b>
          to
          <b>Jan 2nd, 2000 at 12:00 AM</b>
        

      
        <b>Somewhere out there</b>
      
    </p>

    
        <p>Uniform: Black Slacks</p>
      
    
    
      <p>Let&#x27;s all go sing somewhere!</p>
    
  </body>
</html>"
        );
    }
}
