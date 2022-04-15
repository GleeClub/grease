use anyhow::Result;
use time::{Duration, OffsetDateTime};

use crate::db::DbConn;
use crate::models::event::Event;
use crate::models::semester::Semester;
use crate::util::now;

const LIST_ADDRESS: &'static str = "gleeclub@lists.gatech.edu";

pub async fn send_event_emails(events_since: Option<OffsetDateTime>) -> Result<()> {
    let conn = DbConn::connect().await?;
    let current_semester = Semester::get_current(&conn)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to get current semester: {}", err.message))?;
    let all_events = Event::for_semester(&current_semester.name, &conn)
        .await
        .map_err(|err| {
            anyhow::anyhow!(
                "Failed to get events from current semester: {}",
                err.message
            )
        })?;
    let since = events_since.unwrap_or_else(|| now() - Duration::hours(1));

    two_days_out::send_emails(&all_events[..], &since, &conn)
}

mod two_days_out {
    use anyhow::Result;
    use time::{Duration, OffsetDateTime};

    use super::LIST_ADDRESS;
    use crate::db::DbConn;
    use crate::email::Email;
    use crate::models::event::Event;

    pub fn send_emails(all_events: &[Event], since: &OffsetDateTime, conn: &DbConn) -> Result<()> {
        for event in filter_unaddressed_events(all_events, since)? {
            Email {
                subject: email_subject(event),
                to_address: LIST_ADDRESS.to_owned(),
                content: email_body(event, conn)?,
            }
            .send()?;
        }

        Ok(())
    }

    fn filter_unaddressed_events<'e>(
        events: &'e [Event],
        since: &OffsetDateTime,
    ) -> Result<impl Iterator<Item = &'e Event>> {
        let two_days_from_now = crate::util::now() + Duration::days(2);
        let two_days_from_last_checked = *since + Duration::days(2);

        Ok(events.iter().filter(move |event| {
            event.call_time.0 < two_days_from_now && event.call_time.0 > two_days_from_last_checked
        }))
    }

    fn email_subject(event: &Event) -> String {
        format!("{} is in 48 Hours", event.name)
    }

    fn email_body(event: &Event, conn: &DbConn) -> Result<String> {
        Ok(String::new())
        // let url = format!(
        //     "https://gleeclub.gatech.edu/glubhub/#/events/{}",
        //     event.event.id
        // );
        // let format_time = |time: &NaiveDateTime| time.format("").to_string();
        // let uniform = if let Some(uniform) = event.gig.as_ref().map(|gig| &gig.uniform) {
        //     Some(Uniform::load(*uniform, conn)?)
        // } else {
        //     None
        // };

        // Ok(html! {
        //     <div>
        //         <h2>
        //             <a href=url target="_blank">
        //                 { text!("{}", event.event.name) }
        //             </a>
        //         </h2>
        //         <p>
        //             <b>{ text!("{}", event.event.type_) }</b>
        //             ", "
        //             {
        //                 if let Some(release_time) = &event.event.release_time {
        //                     html! {
        //                         <span>
        //                             "from"
        //                             <b> { text!("{}", format_time(&event.event.call_time)) } </b>
        //                             "to"
        //                             <b> { text!("{}", format_time(release_time)) } </b>
        //                         </span>
        //                     }
        //                 } else {
        //                     html! {
        //                         <span>
        //                             <b> { text!("{}", format_time(&event.event.call_time)) } </b>
        //                         </span>
        //                     }
        //                 }
        //             }
        //             {
        //                 if let Some(location) = &event.event.location {
        //                     html! {
        //                         <span>
        //                             "at"
        //                             <b>{ text!("{}", location) }</b>
        //                         </span>
        //                     }
        //                 } else {
        //                     html! {
        //                         <span></span>
        //                     }
        //                 }
        //             }
        //         </p>
        //         {
        //             if let Some(uniform) = uniform {
        //                 html! {
        //                     <p> { text!("Uniform: {}", uniform.name) } </p>
        //                 }
        //             } else {
        //                 html! {
        //                     <p></p>
        //                 }
        //             }
        //         }
        //         {
        //             if let Some(comments) = &event.event.comments {
        //                 html! {
        //                     <p> { text!("{}", comments) } </p>
        //                 }
        //             } else {
        //                 html! {
        //                     <p></p>
        //                 }
        //             }
        //         }
        //     </div>
        // })
    }
}
