use async_graphql::Result;
use sqlx::PgPool;

use crate::email::{Email, MEMBER_LIST_ADDRESS};
use crate::models::event::Event;

pub async fn email_for_event(event: &Event, pool: &PgPool) -> Result<Email<'static>> {
    let subject = format!("{} is in 48 Hours", event.name);
    let body = event_email_body(event, pool).await?;

    Ok(Email {
        subject,
        body,
        address: MEMBER_LIST_ADDRESS,
    })
}

async fn event_email_body(_event: &Event, _pool: &PgPool) -> Result<String> {
    Ok(String::new())

    // let url = format!(
    //     "https://gleeclub.gatech.edu/glubhub/#/events/{}",
    //     event.event.id
    // );
    // let format_time = |time: &NaiveDateTime| time.format("").to_string();
    // let uniform = if let Some(uniform) = event.gig.as_ref().map(|gig| &gig.uniform) {
    //     Some(Uniform::load(*uniform, pool)?)
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
