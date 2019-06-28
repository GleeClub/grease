//! All event-focused routes.

use super::basic_success;
use crate::check_for_permission;
use auth::User;
use db::*;
use error::*;
use pinto::query_builder::Order;
use serde_json::{json, Value};

/// Get a single event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Query Parameters:
///   * full: boolean (*optional*) - Whether to include uniform and attendance.
///
/// ## Return Format:
///
/// If `full = true`, then the format from
/// [to_json_full](../../db/models/event/struct.EventWithGig.html#method.to_json_full)
/// will be returned. Otherwise, the format from
/// [to_json](../../db/models/event/struct.EventWithGig.html#method.to_json)
/// will be returned.
pub fn get_event(event_id: i32, full: Option<bool>, mut user: User) -> GreaseResult<Value> {
    Event::load(event_id, &mut user.conn).and_then(|event_with_gig| {
        if full.unwrap_or(false) {
            event_with_gig.to_json_full(&user.member.member, &mut user.conn)
        } else {
            Ok(event_with_gig.to_json())
        }
    })
}

/// Get all events for the semester.
///
/// ## Query Parameters:
///   * full: boolean (*optional*) - Whether to include uniform and attendance.
///   * event_types: string (*optional*) - A comma-delimited list of event types to
///       filter the events by. If unspecified, simply returns all events.
///
/// ## Return Format:
///
/// Returns a list of `Event`s, ordered by
/// [callTime](../../db/models/struct.Event.html#structfield.call_time).
/// See [get_event](fn.get_event.html) for the format of each individual event.
pub fn get_events(
    full: Option<bool>,
    event_types: Option<String>,
    mut user: User,
) -> GreaseResult<Value> {
    let mut events_with_gigs = Event::load_all_for_current_semester(&mut user.conn)?;
    // if event types are provided,
    if let Some(event_types) = event_types {
        // load all of the event types with those names
        let event_types = event_types
            .split(",")
            .map(|type_| {
                user.conn.first::<EventType>(
                    &EventType::filter(&format!("name = '{}'", type_)),
                    format!("No event type exists named {}.", type_),
                )
            })
            .collect::<GreaseResult<Vec<EventType>>>()?;
        // and filter the events by the provided types
        events_with_gigs.retain(|event| {
            event_types
                .iter()
                .filter(|type_| &event.event.type_ == &type_.name)
                .next()
                .is_some()
        });
    }

    events_with_gigs
        .into_iter()
        .map(|event_with_gig| {
            // include extra data if the `full` parameter is set to `true`
            if full.unwrap_or(false) {
                event_with_gig.to_json_full(&user.member.member, &mut user.conn)
            } else {
                Ok(event_with_gig.to_json())
            }
        })
        .collect::<GreaseResult<Vec<Value>>>()
        .map(|events_with_gigs| events_with_gigs.into())
}

/// Create a new event or events.
///
/// ## Input Format:
///
/// Expects a [NewEvent](../../db/models/struct.NewEvent.html).
///
/// ## Return Format:
///
/// ```json
/// {
///     "id": integer
/// }
/// ```
///
/// Returns an object containing the id of the newly created event
/// (the first one if multiple were created).
pub fn new_event((new_event, mut user): (NewEvent, User)) -> GreaseResult<Value> {
    if !user.has_permission("create-event", None)
        && !user.has_permission("create-event", Some(&new_event.type_))
    {
        Err(GreaseError::Forbidden(Some("create-event".to_owned())))
    } else {
        Event::create(new_event, None, &mut user.conn).map(|new_id| json!({ "id": new_id }))
    }
}

/// Update an existing event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Input Format:
///
/// Expects an [EventUpdate](../../db/models/struct.EventUpdate.html).
pub fn update_event(
    id: i32,
    (updated_event, mut user): (EventUpdate, User),
) -> GreaseResult<Value> {
    Event::update(id, updated_event, &mut user.conn).map(|_| basic_success())
}

/// Delete an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
pub fn delete_event(id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "delete-event");
    Event::delete(id, &mut user.conn).map(|_| basic_success())
}

/// Load the attendance for an event.
///
/// If the current member can edit all attendance, they will be provided with
/// all sections: "Baritone", "Bass", "Tenor 1", "Tenor 2", "Unsorted".
///
/// If they can only edit their own section's attendance, then they will see
/// just their section's attendance (only works for sorted members). Otherwise,
/// anyone else will be denied viewing privileges.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Return Format:
///
/// ```json
/// {
///     "<section>": [
///         {
///             "attendance": Attendance,
///             "member": Member,
///         },
///         ...
///     ],
///     ...
/// }
/// ```
///
/// See the [Attendance](../../db/models/struct.Attendance.html) and the
/// [Member](../../db/models/struct.Member.html) models for how they will be
/// returned.
pub fn get_attendance(id: i32, mut user: User) -> GreaseResult<Value> {
    let event = Event::load(id, &mut user.conn)?;
    let section = event.event.section.as_ref().map(|s| s.as_str());

    if user.has_permission("view-attendance", None) {
        let all_attendance = Attendance::load_for_event_separate_by_section(id, &mut user.conn)?;
        let mut attendance_json = json!({});
        for (section, section_attendance) in all_attendance.into_iter() {
            attendance_json[section] = section_attendance
                .into_iter()
                .map(|(attendance, member_for_semester)| {
                    json!({
                        "attendance": attendance,
                        "member": member_for_semester.member.to_json()
                    })
                })
                .collect::<Vec<_>>()
                .into();
        }
        Ok(attendance_json)
    } else if section.is_some()
        && user.has_permission("view-attendance", Some(event.event.type_.as_str()))
    {
        let section_attendance =
            Attendance::load_for_event_for_section(id, section, &mut user.conn)?;
        Ok(json!({
            section.unwrap_or("Unsorted"): section_attendance
                .into_iter()
                .map(|(attendance, member_for_semester)| json!({
                    "attendance": attendance,
                    "member": member_for_semester.member.to_json()
                }))
                .collect::<Vec<_>>()
        }))
    } else {
        Err(GreaseError::Forbidden(Some("view-attendance".to_owned())))
    }
}

/// Get the attendance for a single member.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
///
/// ## Return Format:
///
/// Returns an [Attendance](../../db/models/struct.Attendance.html#json-format).
pub fn get_member_attendance(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    Attendance::load(&member, event_id, &mut user.conn).map(|attendance| json!(attendance))
}

/// Get the attendance for a member for all events of the current semester.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the requested member
///
/// ## Return Format:
///
/// ```json
/// [
///     {
///         "event": Event,
///         "attendance": Attendance
///     },
///     ...
/// ]
/// ```
///
/// Returns a list of event/attendance pairs, ordered by
/// [callTime](../../db/models/struct.Event.html#structfield.call_time).
/// See [Attendance](../../db/models/struct.Attendance.html#json-format) for the
/// JSON format for the fields.
pub fn get_member_attendance_for_semester(member: String, mut user: User) -> GreaseResult<Value> {
    let current_semester = Semester::load_current(&mut user.conn)?;
    Attendance::load_for_member_at_all_events(&member, &current_semester.name, &mut user.conn).map(
        |event_attendance_pairs| {
            event_attendance_pairs
                .into_iter()
                .map(|(event, attendance)| {
                    json!({
                        "event": event,
                        "attendance": attendance
                    })
                })
                .collect::<Vec<_>>()
                .into()
        },
    )
}

/// Update the attendance for a member at an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
///
/// ## Input Format:
///
/// Expects an [AttendanceForm](../../db/models/struct.AttendanceForm.html).
pub fn update_attendance(
    event_id: i32,
    member: String,
    (mut user, attendance_form): (User, AttendanceForm),
) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    if event.event.section.is_some() {
        if user.has_permission("edit-attendance", None)
            || user.has_permission("edit-attendance", Some(event.event.type_.as_str()))
        {
            Attendance::update(event_id, &member, &attendance_form, &mut user.conn)
                .map(|_| basic_success())
        } else {
            Err(GreaseError::Forbidden(Some("edit-attendance".to_owned())))
        }
    } else {
        check_for_permission!(user => "edit-attendance");
        Attendance::update(event_id, &member, &attendance_form, &mut user.conn)
            .map(|_| basic_success())
    }
}

/// Excuse the absence of all members that didn't confirm attendance to an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
pub fn excuse_unconfirmed_for_event(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Attendance::excuse_unconfirmed(event_id, &mut user.conn).map(|_| basic_success())
}

/// Get a the carpools for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event

/// ## Return Format:
///
/// Returns an [EventCarpool](../../db/models/carpool/struct.EventCarool.html#method.to_json).
pub fn get_carpools(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Carpool::load_for_event(event_id, &mut user.conn).map(|carpools| {
        carpools
            .into_iter()
            .map(|carpool| carpool.to_json())
            .collect::<Vec<_>>()
            .into()
    })
}

/// Update the carpools for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Input Format:
///
/// Returns an [UpdatedCarpool](../../db/models/struct.UpdatedCarpool.html#json-format).
pub fn update_carpools(
    event_id: i32,
    (updated_carpools, mut user): (Vec<UpdatedCarpool>, User),
) -> GreaseResult<Value> {
    Carpool::update_for_event(event_id, updated_carpools, &mut user.conn).map(|_| basic_success())
}

/// Get the setlist for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Return Format:
///
/// Returns a list of [GigSong](../../db/models/struct.GigSong.html) objects.
pub fn get_setlist(event_id: i32, mut user: User) -> GreaseResult<Value> {
    GigSong::load_for_event(event_id, &mut user.conn).map(|setlist| json!(setlist))
}

/// Edit the setlist for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Input Format:
///
/// Expects a list of [NewGigSong](../../db/models/struct.NewGigSong.html) objects
/// in the order that the songs should appear for the setlist.
pub fn edit_setlist(
    event_id: i32,
    (updated_setlist, mut user): (Vec<NewGigSong>, User),
) -> GreaseResult<Value> {
    GigSong::update_for_event(event_id, updated_setlist, &mut user.conn)
        .map(|setlist| json!(setlist))
}

/// Get an absence request for a member from an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
///
/// ## Return Format:
///
/// Returns an [AbsenceRequest](../../db/models/struct.AbsenceRequest.html).
pub fn get_absence_request(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::load(&member, event_id, &mut user.conn).map(|request| json!(request))
}

/// Get all absence requests for the current semester.
///
/// ## Return Format:
///
/// Returns a list of [AbsenceRequest](../../db/models/struct.AbsenceRequest.html)s.
pub fn get_absence_requests(mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::load_all_for_this_semester(&mut user.conn).map(|requests| json!(requests))
}

/// Check if a member is excused from an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
///
/// ## Return Format:
///
/// ```json
/// {
///     "excused": boolean
/// }
/// ```
///
/// Returns an object indicating whether the member has been excused.
pub fn member_is_excused(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::excused_for_event(&member, event_id, &mut user.conn)
        .map(|excused| json!({ "excused": excused }))
}

/// Submit an absence request for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Input Format:
///
/// Expects a [NewAbsenceRequest](../../db/models/struct.NewAbsenceRequest.html).
pub fn submit_absence_request(
    event_id: i32,
    (new_request, mut user): (NewAbsenceRequest, User),
) -> GreaseResult<Value> {
    AbsenceRequest::create(
        &user.member.member.email,
        event_id,
        &new_request.reason,
        &mut user.conn,
    )
    .map(|_| basic_success())
}

/// Approve an absence request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
pub fn approve_absence_request(
    event_id: i32,
    member: String,
    mut user: User,
) -> GreaseResult<Value> {
    AbsenceRequest::approve(&member, event_id, &mut user.conn).map(|_| basic_success())
}

/// Deny an absence request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
pub fn deny_absence_request(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::deny(&member, event_id, &mut user.conn).map(|_| basic_success())
}

/// Get all event types.
///
/// ## Return Format:
///
/// Returns a list of [EventType](../../db/models/struct.EventType.html)s.
pub fn get_event_types(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<EventType>(&EventType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

/// Get all section types.
///
/// ## Return Format:
///
/// Returns a list of [SectionTypes](../../db/models/struct.SectionTypes.html)s.
pub fn get_section_types(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<SectionType>(&SectionType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

/// Get a single gig request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
///
/// ## Return Format:
///
/// Returns a [GigRequest](../../db/models/struct.GigRequest.html).
pub fn get_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::load(request_id, &mut user.conn).map(|request| json!(request))
}

/// Get all gig requests.
///
/// ## Query Parameters:
///   * all: boolean (*optional*) - Whether to load all gig requests ever.
///
/// ## Return Format:
///
/// By default, all [GigRequest](../../db/models/struct.GigRequest.html)s
/// that are either from this semester or are still pending from other semesters
/// are returned in a list ordered by
/// [time](../../db/models/struct.GigRequest.html#structfield.time).
/// If `all = true`, then simply all gig requests ever placed are loaded.
pub fn get_gig_requests(all: Option<bool>, mut user: User) -> GreaseResult<Value> {
    let gig_requests = if all.unwrap_or(false) {
        GigRequest::load_all(&mut user.conn)
    } else {
        GigRequest::load_all_for_semester_and_pending(&mut user.conn)
    };

    gig_requests.map(|requests| json!(requests))
}

/// Submit a new gig request.
///
/// ## Input Format:
///
/// Expects a [NewGigRequest](../../db/models/struct.NewGigRequest.html).
///
/// ## Return Format:
///
/// ```json
/// {
///     "id": integer
/// }
/// ```
///
/// Returns an object containing the id of the newly created gig request.
pub fn new_gig_request((new_request, mut conn): (NewGigRequest, DbConn)) -> GreaseResult<Value> {
    new_request
        .insert_returning_id(&mut conn)
        .map(|new_id| json!({ "id": new_id }))
}

/// Dismiss a gig request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
pub fn dismiss_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::set_status(request_id, GigRequestStatus::Dismissed, &mut user.conn)
        .map(|_| basic_success())
}

/// Re-open a gig request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
pub fn reopen_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::set_status(request_id, GigRequestStatus::Pending, &mut user.conn)
        .map(|_| basic_success())
}

/// Create a new event or events.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
///
/// ## Input Format:
///
/// Expects a [GigRequestForm](../../db/models/struct.GigRequestForm.html).
///
/// ## Return Format:
///
/// ## Return Format:
///
/// ```json
/// {
///     "id": integer
/// }
/// ```
///
/// Returns an object containing the id of the newly created event
/// (the first one if multiple were created).
pub fn create_event_from_gig_request(
    request_id: i32,
    (form, mut user): (GigRequestForm, User),
) -> GreaseResult<Value> {
    let request = GigRequest::load(request_id, &mut user.conn)?;
    if request.status != GigRequestStatus::Pending {
        Err(GreaseError::BadRequest(
            "The gig request must be pending to create an event for it.".to_owned(),
        ))
    } else {
        Event::create(form.event, Some((request, form.gig)), &mut user.conn)
            .map(|new_id| json!({ "id": new_id }))
    }
}
