use crate::db::{connect_to_db, Event};
use chrono::{Duration, Local, NaiveDateTime};
use error::GreaseResult;

const LIST_ADDRESS: &'static str = "gleeclub@lists.gatech.edu";

pub fn send_event_emails(events_since: Option<NaiveDateTime>) -> GreaseResult<()> {
    let conn = connect_to_db()?;
    let all_events = Event::load_all_for_current_semester(&conn)?;
    let since = events_since.unwrap_or_else(|| Local::now().naive_local() - Duration::hours(1));

    two_days_out::send_emails(&all_events, &since, &conn)
}

mod two_days_out {
    use super::LIST_ADDRESS;
    use crate::db::{models::event::EventWithGig, Uniform};
    use crate::{error::GreaseResult, util::Email};
    use chrono::{Duration, Local, NaiveDateTime};
    use diesel::MysqlConnection;

    pub fn send_emails(
        all_events: &Vec<EventWithGig>,
        since: &NaiveDateTime,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        for event in filter_unaddressed_events(all_events, since) {
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
        events: &'e Vec<EventWithGig>,
        since: &NaiveDateTime,
    ) -> impl Iterator<Item = &'e EventWithGig> {
        let now = Local::now().naive_local();
        let two_days_from_now = now + Duration::days(2);
        let two_days_from_last_checked = *since + Duration::days(2);

        events.iter().filter(move |event| {
            event.event.call_time < two_days_from_now
                && event.event.call_time > two_days_from_last_checked
        })
    }

    fn email_subject(event: &EventWithGig) -> String {
        format!("{} is in 48 Hours", event.event.name)
    }

    fn email_body(event: &EventWithGig, conn: &MysqlConnection) -> GreaseResult<String> {
        let url = format!(
            "https://gleeclub.gatech.edu/glubhub/#/events/{}",
            event.event.id
        );
        let format_time = |time: &NaiveDateTime| time.format("").to_string();
        let uniform = if let Some(uniform) = event.gig.as_ref().map(|gig| &gig.uniform) {
            Some(Uniform::load(*uniform, conn)?)
        } else {
            None
        };

        Ok("".to_owned())

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
