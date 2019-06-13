use app_route::AppRoute;
use super::{OptionalIdQuery, OptionalEmailQuery};
use crate::check_for_permission;
use db::models::*;
use db::models::event::EventWithGig;
use auth::User;
use error::{GreaseError, GreaseResult};
use serde_json::{Value, json};

#[derive(AppRoute, Debug)]
#[route("/events")]
pub struct EventsRequest {
    #[query]
    pub query: OptionalIdQuery,
}

pub fn get_events(req: EventsRequest, user: User) -> GreaseResult<Value> {
    let event_to_json = |event: Event, gig: Option<Gig>| json!({
            "id": event.id,
            "name": event.name,
            "semester": event.semester,
            "type": event.type_,
            "call_time": event.call_time,
            "release_time": event.release_time,
            "points": event.points,
            "comments": event.comments,
            "location": event.location,
            "gig_count": event.gig_count,
            "default_attend": event.default_attend,
            "section": event.section,
            "performance_time": gig.as_ref().map(|gig| gig.performance_time),
            "uniform": gig.as_ref().map(|gig| &gig.uniform),
            "contact_name": gig.as_ref().map(|gig| &gig.contact_name),
            "contact_email": gig.as_ref().map(|gig| &gig.contact_email),
            "contact_phone": gig.as_ref().map(|gig| &gig.contact_phone),
            "price": gig.as_ref().map(|gig| gig.price),
            "public": gig.as_ref().map(|gig| gig.public),
            "summary": gig.as_ref().map(|gig| &gig.summary),
            "description": gig.as_ref().map(|gig| &gig.description),
        });
    if let Some(event_id) = req.query.id {
        let EventWithGig { event, gig } = Event::load(event_id, &user.conn)?;
        Ok(event_to_json(event, gig))
    } else {
        Event::load_all(&user.conn)
            .map(|events_with_gigs| json!(events_with_gigs
                .into_iter()
                .map(|EventWithGig { event, gig }| event_to_json(event, gig))
                .collect::<Vec<Value>>()
            ))
    }
}

// TODO: new events
// TODO: edit events

// Attendance
// RidesIn
// Carpool
// Event
// Gig
// GigRequest
// AbsenceRequest
// EventType
// GigSong

// SectionType

// Uniform
