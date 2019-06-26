use super::basic_success;
use crate::check_for_permission;
use auth::User;
use db::models::attendance::AttendanceForm;
use db::*;
use error::*;
use pinto::query_builder::Order;
use serde_json::{json, Value};

pub fn get_event(event_id: i32, full: Option<bool>, mut user: User) -> GreaseResult<Value> {
    Event::load(event_id, &mut user.conn).and_then(|event_with_gig| {
        if full.unwrap_or(false) {
            event_with_gig.to_json_full(&user.member.member, &mut user.conn)
        } else {
            Ok(event_with_gig.to_json())
        }
    })
}

pub fn get_events(
    full: Option<bool>,
    event_type: Option<String>,
    mut user: User,
) -> GreaseResult<Value> {
    let events_with_gigs = if let Some(event_type) = event_type.filter(|type_| type_.len() > 0) {
        let event_type = user.conn.first::<EventType>(
            &EventType::filter(&format!("name = '{}'", event_type)),
            format!("No event type exists named {}.", event_type),
        )?;
        Event::load_all_of_type_for_current_semester(&event_type.name, &mut user.conn)
    } else {
        Event::load_all_for_current_semester(&mut user.conn)
    };

    events_with_gigs.and_then(|events_with_gigs| {
        events_with_gigs
            .into_iter()
            .map(|event_with_gig| {
                if full.unwrap_or(false) {
                    event_with_gig.to_json_full(&user.member.member, &mut user.conn)
                } else {
                    Ok(event_with_gig.to_json())
                }
            })
            .collect::<GreaseResult<Vec<Value>>>()
            .map(|events_with_gigs| events_with_gigs.into())
    })
}

pub fn new_event((new_event, mut user): (NewEvent, User)) -> GreaseResult<Value> {
    Event::create(new_event, None, &mut user.conn).map(|new_id| json!({ "id": new_id }))
}

pub fn update_event(
    event_id: i32,
    (updated_event, mut user): (EventUpdate, User),
) -> GreaseResult<Value> {
    Event::update(event_id, updated_event, &mut user.conn).map(|_| basic_success())
}

pub fn delete_event(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Event::delete(event_id, &mut user.conn).map(|_| basic_success())
}

pub fn get_attendance(event_id: i32, mut user: User) -> GreaseResult<Value> {
    let event = Event::load(event_id, &mut user.conn)?;
    let section = event.event.section.as_ref().map(|s| s.as_str());

    if user.has_permission("view-attendance", None) {
        let all_attendance =
            Attendance::load_for_event_separate_by_section(event_id, &mut user.conn)?;
        let mut attendance_json = json!({});
        for (section, section_attendance) in all_attendance.into_iter() {
            let section = section.unwrap_or("unsorted".to_owned());
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
            Attendance::load_for_event_for_section(event_id, section, &mut user.conn)?;
        Ok(json!({
            section.unwrap_or("unsorted"): section_attendance
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

pub fn get_member_attendance(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    Attendance::load(&member, event_id, &mut user.conn).map(|attendance| json!(attendance))
}

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

pub fn excuse_unconfirmed_for_event(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Attendance::excuse_unconfirmed(event_id, &mut user.conn).map(|_| basic_success())
}

pub fn get_carpools(event_id: i32, mut user: User) -> GreaseResult<Value> {
    Carpool::load_for_event(event_id, &mut user.conn).map(|carpools| {
        carpools
            .into_iter()
            .map(|carpool| carpool.to_json())
            .collect::<Vec<_>>()
            .into()
    })
}

pub fn update_carpools(
    event_id: i32,
    (updated_carpools, mut user): (Vec<UpdatedCarpool>, User),
) -> GreaseResult<Value> {
    Carpool::update_for_event(event_id, updated_carpools, &mut user.conn).map(|_| basic_success())
}

pub fn get_setlist(event_id: i32, mut user: User) -> GreaseResult<Value> {
    GigSong::load_for_event(event_id, &mut user.conn).map(|setlist| json!(setlist))
}

pub fn edit_setlist(
    event_id: i32,
    (updated_setlist, mut user): (Vec<NewGigSong>, User),
) -> GreaseResult<Value> {
    GigSong::update_for_event(event_id, updated_setlist, &mut user.conn)
        .map(|setlist| json!(setlist))
}

pub fn get_absence_request(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::load(&member, event_id, &mut user.conn).map(|request| json!(request))
}

pub fn get_absence_requests(mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::load_all_for_this_semester(&mut user.conn).map(|requests| json!(requests))
}

pub fn member_is_excused(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::excused_for_event(&member, event_id, &mut user.conn)
        .map(|excused| json!({ "excused": excused }))
}

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

pub fn approve_absence_request(
    event_id: i32,
    member: String,
    mut user: User,
) -> GreaseResult<Value> {
    AbsenceRequest::approve(&member, event_id, &mut user.conn).map(|_| basic_success())
}

pub fn deny_absence_request(event_id: i32, member: String, mut user: User) -> GreaseResult<Value> {
    AbsenceRequest::deny(&member, event_id, &mut user.conn).map(|_| basic_success())
}

pub fn get_event_types(mut user: User) -> GreaseResult<Value> {
    user.conn.load::<EventType>(&EventType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

pub fn get_section_types(mut user: User) -> GreaseResult<Value> {
    user.conn.load::<SectionType>(&SectionType::select_all_in_order("name", Order::Asc))
        .map(|types| json!(types))
}

pub fn get_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::load(request_id, &mut user.conn).map(|request| json!(request))
}

pub fn get_gig_requests(all: Option<bool>, mut user: User) -> GreaseResult<Value> {
    let gig_requests = if all.unwrap_or(false) {
        GigRequest::load_all(&mut user.conn)
    } else {
        GigRequest::load_all_for_semester_and_pending(&mut user.conn)
    };

    gig_requests.map(|requests| json!(requests))
}

pub fn new_gig_request((new_request, mut conn): (NewGigRequest, DbConn)) -> GreaseResult<Value> {
    new_request.insert_returning_id(&mut conn)
        .map(|new_id| json!({ "id": new_id }))
}

pub fn dismiss_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::set_status(request_id, GigRequestStatus::Dismissed, &mut user.conn)
        .map(|_| basic_success())
}

pub fn reopen_gig_request(request_id: i32, mut user: User) -> GreaseResult<Value> {
    GigRequest::set_status(request_id, GigRequestStatus::Pending, &mut user.conn)
        .map(|_| basic_success())
}

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

// GigRequest