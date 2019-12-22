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
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// If `full = true`, then the format from
/// [to_json_full](crate::db::models::event::EventWithGig::to_json_full)
/// will be returned. Otherwise, the format from
/// [to_json](crate::db::models::event::EventWithGig::to_json)
/// will be returned.
pub fn get_event(event_id: i32, full: Option<bool>, mut user: User) -> GreaseResult<Value> {
    Event::load(event_id, &mut user.conn).and_then(|event_with_gig| {
        if full.unwrap_or(false) {
            let attendance = Attendance::load(&user.member.member.email, event_id, &mut user.conn)?;
            event_with_gig.to_json_full(
                attendance.as_ref(),
                user.member.active_semester.is_some(),
                &mut user.conn,
            )
        } else {
            Ok(json!(event_with_gig))
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
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [Event](crate::db::models::Event)s, ordered by
/// [callTime](crate::db::models::Event#structfield.call_time).
/// See [get_event](crate::routes::event_routes::get_event) for the format of each individual event.
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

    if full.unwrap_or(false) {
        let grades = if let Some(active_semester) = &user.member.active_semester {
            user.member
                .member
                .calc_grades(active_semester, &mut user.conn)?
                .changes
        } else {
            Vec::new()
        };
        events_with_gigs
            .into_iter()
            .zip(grades.into_iter().map(Some).chain(std::iter::repeat(None)))
            .map(|(event_with_gig, grade_change)| {
                event_with_gig.to_json_with_grade_change(
                    grade_change.as_ref(),
                    user.member.active_semester.is_some(),
                    &mut user.conn,
                )
            })
            .collect::<GreaseResult<Vec<Value>>>()
            .map(|events_with_gigs| events_with_gigs.into())
    } else {
        Ok(json!(events_with_gigs))
    }
}

/// Create a new event or events.
///
/// ## Input Format:
///
/// Expects a [NewEvent](crate::db::models::NewEvent).
///
/// ## Required Permissions:
///
/// The user must be logged in, and must be able to either
/// "create-event" generally or "create-event" for the specified type.
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
    check_for_permission!(user => "create-event", &new_event.type_);
    Event::create(new_event, None, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

/// Update an existing event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in, and must be able to either
/// "edit-all-events" generally or "modify-event" of the specified type.
///
/// ## Input Format:
///
/// Expects an [EventUpdate](crate::db::models::EventUpdate).
pub fn update_event(
    id: i32,
    (updated_event, mut user): (EventUpdate, User),
) -> GreaseResult<Value> {
    if !user.has_permission("edit-all-events", None) {
        let event = Event::load(id, &mut user.conn)?;
        check_for_permission!(user => "modify-event", &event.event.type_);
    }

    Event::update(id, updated_event, &mut user.conn).map(|_| basic_success())
}

/// RSVP for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in and active for the semester of the event.
pub fn rsvp_for_event(id: i32, attending: bool, mut user: User) -> GreaseResult<Value> {
    Event::rsvp(id, &user.member, attending, &mut user.conn).map(|_| basic_success())
}

/// Delete an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in, and must be able to "delete-event"
/// generally or for the specified event's type.
pub fn delete_event(id: i32, mut user: User) -> GreaseResult<Value> {
    let event = Event::load(id, &mut user.conn)?;
    check_for_permission!(user => "delete-event", &event.event.type_);

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
/// ## Required Permissions:
///
/// The user must be logged in. To view all attendance, a user must be
/// able to "view-attendance" generally. If they can't, they must be
/// able to "view-attendance-own-section" for their own section.
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
/// See the [Attendance](crate::db::models::Attendance) and the
/// [Member](crate::db::models::Member) models for how they will be
/// returned.
pub fn get_attendance(id: i32, mut user: User) -> GreaseResult<Value> {
    let event = Event::load(id, &mut user.conn)?;
    let member_section = user
        .member
        .active_semester
        .as_ref()
        .and_then(|active_semester| {
            active_semester
                .section
                .as_ref()
                .map(|section| section.as_str())
        });

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
    } else if member_section.is_some()
        && user.has_permission(
            "view-attendance-own-section",
            Some(event.event.type_.as_str()),
        )
    {
        let section_attendance =
            Attendance::load_for_event_for_section(id, member_section, &mut user.conn)?;
        Ok(json!({
            member_section.unwrap_or("Unsorted"): section_attendance
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
/// ## Required Permissions:
///
/// The user must be logged in. To view another member's attendance, the user must be
/// able to "view-attendance" generally or for the given event's type.
///
/// ## Return Format:
///
/// Returns an [Attendance](crate::db::models::Attendance).
pub fn get_member_attendance(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    if &member != &user.member.member.email {
        let event = Event::load(event_id, &mut user.conn)?;
        check_for_permission!(user => "view-attendance", &event.event.type_);
    }

    Attendance::load(&member, event_id, &mut user.conn).map(|attendance| json!(attendance))
}

// TODO: fix these docs

/// Get the attendance of all active members for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// ```json
/// [
///     {
///         "attendance": Attendance,
///         "member": Member
///     },
///     ...
/// ]
/// ```
///
/// Returns a list of JSON objects with [Attendance](crate::db::models::Attendance)s
/// and [MemberForSemester](crate::db::models::member::MemberForSemester)s.
pub fn see_whos_attending(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Attendance::load_for_event(event_id, &mut user.conn).map(|attendance| {
        attendance
            .into_iter()
            .map(|(attendance, member_for_semester)| {
                let mut member = member_for_semester.to_json();
                member["shouldAttend"] = json!(attendance.should_attend);
                member["didAttend"] = json!(attendance.did_attend);
                member["confirmed"] = json!(attendance.confirmed);
                member["minutesLate"] = json!(attendance.minutes_late);
                member
            })
            .collect::<Vec<Value>>()
            .into()
    })
}

/// Get the attendance for a member for all events of the current semester.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the requested member
///
/// ## Required Permissions:
///
/// The user must be logged in. To view another member's attendance,
/// the user must be able to "view-attendance" generally.
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
/// [callTime](crate::db::models::Event#structfield.call_time).
/// See [Attendance](crate::db::models::Attendance#json-format) for the
/// JSON format for the fields.
pub fn get_member_attendance_for_semester(member: String, mut user: User) -> GreaseResult<Value> {
    if &member != &user.member.member.email {
        check_for_permission!(user => "view-attendance");
    }

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
/// ## Required Permissions:
///
/// The user must be logged in. To edit another member's attendance,
/// the user must be able to either "edit-attendance" generally or for
/// the given event's type, or "edit-attendance-own-section" generally
/// or for the given event's type only when the current user is in the
/// same section as the member whose attendance they are editing.
///
/// ## Input Format:
///
/// Expects an [AttendanceForm](crate::db::models::AttendanceForm).
pub fn update_attendance(
    event_id: i32,
    member: String,
    (mut user, attendance_form): (User, AttendanceForm),
) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    let user_section = user
        .member
        .active_semester
        .as_ref()
        .and_then(|active_semester| active_semester.section.clone());
    let member_section = ActiveSemester::load(&member, &event.event.semester, &mut user.conn)?
        .and_then(|active_semester| active_semester.section);

    if user.has_permission("edit-attendance", Some(event.event.type_.as_str()))
        || (user_section == member_section
            && user.has_permission(
                "edit-attendance-own-section",
                Some(event.event.type_.as_str()),
            ))
    {
        Attendance::update(event_id, &member, &attendance_form, &mut user.conn)
            .map(|_| basic_success())
    } else {
        Err(GreaseError::Forbidden(Some("edit-attendance".to_owned())))
    }
}

/// Excuse the absence of all members that didn't confirm attendance to an event.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-attendance"
/// either generally or for the given event's type.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
pub fn excuse_unconfirmed_for_event(event_id: i32, mut user: User) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    check_for_permission!(user => "edit-attendance", event.event.type_.as_str());

    Attendance::excuse_unconfirmed(event_id, &mut user.conn).map(|_| basic_success())
}

/// Get a the carpools for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns an [EventCarpool](crate::db::models::carpool::EventCarpool#method.to_json).
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-carpool"
/// either generally or for the given event's type.
///
/// ## Input Format:
///
/// Returns an [UpdatedCarpool](crate::db::models::UpdatedCarpool).
pub fn update_carpools(
    event_id: i32,
    (updated_carpools, mut user): (Vec<UpdatedCarpool>, User),
) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    check_for_permission!(user => "edit-carpool", &event.event.type_.as_str());

    Carpool::update_for_event(event_id, updated_carpools, &mut user.conn).map(|_| basic_success())
}

/// Get the setlist for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in  .
///
/// ## Return Format:
///
/// Returns a list of [Song](crate::db::models::Song)s.
pub fn get_setlist(event_id: i32, mut user: User) -> GreaseResult<Value> {
    GigSong::load_for_event(event_id, &mut user.conn).map(|setlist| json!(setlist))
}

/// Edit the setlist for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-setlist"
/// either generally or for the given event's type.
///
/// ## Input Format:
///
/// Expects a list of [NewGigSong](crate::db::models::NewGigSong)s
/// in the order that the songs should appear for the setlist.
pub fn edit_setlist(
    event_id: i32,
    (updated_setlist, mut user): (Vec<NewGigSong>, User),
) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    check_for_permission!(user => "edit-carpool", &event.event.type_.as_str());

    GigSong::update_for_event(event_id, updated_setlist, &mut user.conn)
        .map(|setlist| json!(setlist))
}

/// Check for an absence request for the current member from an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns an [AbsenceRequest](crate::db::models::AbsenceRequest).
pub fn get_absence_request(event_id: i32, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::load(&user.member.member.email, event_id, &mut user.conn)
        .map(|request| json!(request))
}

/// Get all absence requests for the current semester.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Return Format:
///
/// Returns a list of [AbsenceRequest](crate::db::models::AbsenceRequest)s.
pub fn get_absence_requests(mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    AbsenceRequest::load_all_for_this_semester(&mut user.conn).map(|requests| json!(requests))
}

/// Check if the current member is excused from an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Required Permissions:
///
/// The user must be logged in.
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
pub fn member_is_excused(event_id: i32, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::excused_for_event(&user.member.member.email, event_id, &mut user.conn)
        .map(|excused| json!({ "excused": excused }))
}

/// Submit an absence request for an event.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///
/// ## Input Format:
///
/// Expects a [NewAbsenceRequest](crate::db::models::NewAbsenceRequest).
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
pub fn approve_absence_request(
    event_id: i32,
    member: String,
    mut user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    AbsenceRequest::approve(&member, event_id, &mut user.conn).map(|_| basic_success())
}

/// Deny an absence request.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the event
///   * member: string (*required*) - The email of the requested member
pub fn deny_absence_request(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    AbsenceRequest::deny(&member, event_id, &mut user.conn).map(|_| basic_success())
}

/// Get all event types.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [EventType](crate::db::models::EventType)s.
pub fn get_event_types(mut user: User) -> GreaseResult<Value> {
    user.conn
        .load::<EventType>(&EventType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

/// Get all section types.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Return Format:
///
/// Returns a list of [SectionType](crate::db::models::SectionType)s.
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Return Format:
///
/// Returns a [GigRequest](crate::db::models::GigRequest).
pub fn get_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    GigRequest::load(request_id, &mut user.conn).map(|request| json!(request))
}

/// Get all gig requests.
///
/// ## Query Parameters:
///   * all: boolean (*optional*) - Whether to load all gig requests ever.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Return Format:
///
/// By default, all [GigRequest](crate::db::models::GigRequest)s
/// that are either from this semester or are still pending from other semesters
/// are returned in a list ordered by
/// [time](crate::db::models::GigRequest#structfield.time).
/// If `all = true`, then simply all gig requests ever placed are loaded.
pub fn get_gig_requests(all: Option<bool>, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
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
/// Expects a [NewGigRequest](crate::db::models::NewGigRequest).
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
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
pub fn dismiss_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    GigRequest::set_status(request_id, GigRequestStatus::Dismissed, &mut user.conn)
        .map(|_| basic_success())
}

/// Re-open a gig request.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
pub fn reopen_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "process-gig-requests");
    GigRequest::set_status(request_id, GigRequestStatus::Pending, &mut user.conn)
        .map(|_| basic_success())
}

/// Create a new event or events from a gig request.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The ID of the gig request
///
/// ## Input Format:
///
/// Expects a [GigRequestForm](crate::db::models::GigRequestForm).
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "process-gig-requests" generally.
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
    check_for_permission!(user => "process-gig-requests");
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
