//! All routes for the API.
//!
//! Check the root of the crate for the full API layout.

pub mod event_routes;
pub mod member_routes;
pub mod misc_routes;
pub mod officer_routes;
pub mod repertoire_routes;
#[macro_use]
pub mod router;

use crate::auth::User;
use backtrace::Backtrace;
use cgi::http::{
    self,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    response,
};
use chrono::{Local, NaiveDateTime, TimeZone};
use error::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

/// The main entry-point for the whole crate.
///
/// Using the [cgi](cgi) crate, requests come in to `stdin` as a stream of
/// bytes, and all headers are set using environment variables. The [cgi](cgi)
/// crate handles putting this into a [Request](http::request::Request) from
/// the [http](http) crate for us.
///
/// This method returns all data as "application/json" responses. On success,
/// a 200 status code is returned, while all errors return appropriate error
/// status codes (See [GreaseError](crate::error::GreaseError) for how those
/// get mapped).
///
/// In the rare case that a `panic!` occurs, this function will attempt
/// to catch it, log it with [log_panic](crate::util::log_panic), and then
/// return a JSON object with some debug information.
pub fn handle_request(request: cgi::Request) -> cgi::Response {
    let mut response = None;
    let bt = Arc::new(Mutex::new(None));

    let bt2 = bt.clone();
    std::panic::set_hook(Box::new(move |_| {
        *bt2.lock().unwrap() = Some(Backtrace::new());
    }));

    panic::catch_unwind(AssertUnwindSafe(|| {
        if request.method() == "OPTIONS" {
            response = Some(options_response());
            return;
        }

        let (status_code, value) = match route_request(&request) {
            Ok(resp) => (200, resp),
            Err(error) => error.as_response(),
        };

        let body = serde_json::to_string(&value)
            .unwrap_or_default()
            .into_bytes();
        response = Some(
            response::Builder::new()
                .status(status_code)
                .header(CONTENT_TYPE, "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .header(CONTENT_LENGTH, body.len().to_string().as_str())
                .body(body)
                .unwrap(),
        );
    }))
    .ok();

    response.unwrap_or_else(move || {
        let error = bt
            .lock()
            .unwrap()
            .as_ref()
            .map(|bt| format!("{:?}", bt))
            .unwrap_or_default();
        crate::util::log_panic(&request, error)
    })
}

pub fn options_response() -> http::Response<Vec<u8>> {
    response::Builder::new()
        .status(200)
        .header("Allow", "GET, POST, DELETE, OPTIONS")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "token,access-control-allow-origin,content-type",
        )
        .body("OK".to_owned().into_bytes())
        .unwrap()
}

fn parse_body<'de, T: Deserialize<'de>>(body: &'de [u8]) -> GreaseResult<T> {
    serde_json::from_slice(body).map_err(|err| GreaseError::BadRequest(err.to_string()))
}

/// Handles routing of incoming requests.
///
/// See the root of the crate for the API layout and
/// [router](router/macro.router.html) for the way this function works.
pub fn route_request(request: &cgi::Request) -> GreaseResult<Value> {
    use routes::event_routes::*;
    use routes::member_routes::*;
    use routes::misc_routes::*;
    use routes::officer_routes::*;
    use routes::repertoire_routes::*;

    let load_user = || crate::auth::User::from_request(request);

    router!(request,
        // authorization
        (POST) [/login] =>
            || login(parse_body(&request.body())?),

        (GET) [/logout] =>
            || logout(load_user()?),

        (POST) [/forgot_password/(email: String)] =>
            |email| forgot_password(email),

        (POST) [/reset_password?(token: String)] =>
            |token| reset_password(token, parse_body(&request.body())?),

        // members
        (GET) [/user] =>
            || get_current_user(load_user()),

        (GET) [/members/(email: String)?(grades: bool)?(details: bool)] =>
            |email, grades, details| get_member(email, grades, details, load_user()?),

        (GET) [/members?(grades: bool)?(include: String)] =>
            |grades, include| get_members(grades, include, load_user()?),

        (POST) [/members] =>
            || new_member(parse_body(&request.body())?),

        (POST) [/members/confirm] =>
            || confirm_for_semester(parse_body(&request.body())?, load_user()?),

        (POST) [/members/(email: String)/(semester: String)] =>
            |email, semester| update_member_semester(email, semester, parse_body(&request.body())?, load_user()?),

        (DELETE) [/members/(email: String)/(semester: String)] =>
            |email, semester| mark_member_inactive_for_semester(email, semester, load_user()?),

        (POST) [/members] =>
            || new_member(parse_body(&request.body())?),

        (POST) [/members/profile] =>
            || update_member_profile(parse_body(&request.body())?, load_user()?),

        (POST) [/members/(email: String)] =>
            |email| update_member_as_officer(email, parse_body(&request.body())?, load_user()?),

        (GET) [/members/(email: String)/login_as] =>
            |email| login_as_member(email, load_user()?),

        (DELETE) [/members/(email: String)?(confirm: bool)] =>
            |email, confirm| delete_member(email, confirm, load_user()?),

        // events
        (GET) [/events/(id: i32)] =>
            |id| get_event(id, load_user()?),

        (GET) [/events?(full: bool)] =>
            |full| get_events(full, load_user()?),

        (POST) [/events] =>
            || new_event(parse_body(&request.body())?, load_user()?),

        (POST) [/events/(id: i32)] =>
            |id| update_event(id, parse_body(&request.body())?, load_user()?),

        (DELETE) [/events/(id: i32)] =>
            |id| delete_event(id, load_user()?),

        (GET) [/public_events] =>
            || get_public_events(),

        (GET) [/week_of_events] =>
            || get_weeks_events(),

        // event details
        (GET) [/events/(id: i32)/attendance] =>
            |id| get_attendance(id, load_user()?),

        (GET) [/events/(id: i32)/see_whos_attending] =>
            |id| see_whos_attending(id, load_user()?),

        (GET) [/events/(id: i32)/attendance/(member: String)] =>
            |id, email| get_member_attendance(id, email, load_user()?),

        (POST) [/events/(id: i32)/attendance/(member: String)] =>
            |id, member| update_attendance(id, member, parse_body(&request.body())?, load_user()?),

        (POST) [/events/(id: i32)/rsvp/(attending: bool)] =>
            |id, attending| rsvp_for_event(id, attending, load_user()?),

        (POST) [/events/(id: i32)/confirm] =>
            |id| confirm_for_event(id, load_user()?),

        (POST) [/events/(id: i32)/attendance/excuse_unconfirmed] =>
            |id| excuse_unconfirmed_for_event(id, load_user()?),

        (GET) [/events/(id: i32)/carpools] =>
            |id| get_carpools(id, load_user()?),

        (POST) [/events/(id: i32)/carpools] =>
            |id| update_carpools(id, parse_body(&request.body())?, load_user()?),

        (GET) [/events/(id: i32)/setlist] =>
            |id| get_setlist(id, load_user()?),

        (POST) [/events/(id: i32)/setlist] =>
            |id| edit_setlist(id, parse_body(&request.body())?, load_user()?),

        // absence requests
        (GET) [/absence_requests] =>
            || get_absence_requests(load_user()?),

        (GET) [/absence_requests/(event_id: i32)] =>
            |id| get_absence_request(id, load_user()?),

        (GET) [/absence_requests/(event_id: i32)/is_excused] =>
            |id| member_is_excused(id, load_user()?),

        (POST) [/absence_requests/(event_id: i32)/(member: String)/approve] =>
            |id, member| approve_absence_request(id, member, load_user()?),

        (POST) [/absence_requests/(event_id: i32)/(member: String)/deny] =>
            |id, member| deny_absence_request(id, member, load_user()?),

        (POST) [/absence_requests/(event_id: i32)] =>
            |id| submit_absence_request(id, parse_body(&request.body())?, load_user()?),

        // gig requests
        (GET) [/gig_requests/(id: i32)] =>
            |id| get_gig_request(id, load_user()?),

        (GET) [/gig_requests?(all: bool)] =>
            |all| get_gig_requests(all, load_user()?),

        (POST) [/gig_requests] =>
            || new_gig_request(parse_body(&request.body())?),

        (POST) [/gig_requests/(id: i32)/dismiss] =>
            |id| dismiss_gig_request(id, load_user()?),

        (POST) [/gig_requests/(id: i32)/reopen] =>
            |id| reopen_gig_request(id, load_user()?),

        (POST) [/gig_requests/(id: i32)/create_event] =>
            |id| create_event_from_gig_request(id, parse_body(&request.body())?, load_user()?),

        // variables
        (GET) [/variables/(key: String)] =>
            |key| get_variable(key, load_user()?),

        (POST) [/variables/(key: String)] =>
            |key| set_variable(key, parse_body(&request.body())?, load_user()?),

        (DELETE) [/variables/(key: String)] =>
            |key| unset_variable(key, load_user()?),

        // announcements
        (GET) [/announcements/(id: i32)] =>
            |id| get_announcement(id, load_user()?),

        (GET) [/announcements?(all: bool)] =>
            |all| get_announcements(all, load_user()?),

        (POST) [/announcements] =>
            || make_new_announcement(parse_body(&request.body())?, load_user()?),

        (POST) [/announcements/(id: i32)/archive] =>
            |id| archive_announcement(id, load_user()?),

        // google docs
        (GET) [/google_docs/(name: String)] =>
            |name| get_google_doc(name, load_user()?),

        (GET) [/google_docs] =>
            || get_google_docs(load_user()?),

        (POST) [/google_docs] =>
            || new_google_doc(parse_body(&request.body())?, load_user()?),

        (POST) [/google_docs/(name: String)] =>
            |name| modify_google_doc(name, parse_body(&request.body())?, load_user()?),

        (DELETE) [/google_docs/(name: String)] =>
            |name| delete_google_doc(name, load_user()?),

        // meeting minutes
        (GET) [/meeting_minutes/(id: i32)] =>
            |id| get_meeting_minutes(id, load_user()?),

        (GET) [/meeting_minutes] =>
            || get_all_meeting_minutes(load_user()?),

        (POST) [/meeting_minutes] =>
            || new_meeting_minutes(parse_body(&request.body())?, load_user()?),

        (POST) [/meeting_minutes/(id: i32)] =>
            |id| modify_meeting_minutes(id, parse_body(&request.body())?, load_user()?),

        (GET) [/meeting_minutes/(id: i32)/email] =>
            |id| send_minutes_as_email(id, load_user()?),

        (DELETE) [/meeting_minutes/(id: i32)] =>
            |id| delete_meeting_minutes(id, load_user()?),

        // uniforms
        (GET) [/uniforms/(id: i32)] =>
            |id| get_uniform(id, load_user()?),

        (GET) [/uniforms] =>
            || get_uniforms(load_user()?),

        (POST) [/uniforms] =>
            || new_uniform(parse_body(&request.body())?, load_user()?),

        (POST) [/uniforms/(id: i32)] =>
            |id| modify_uniform(id, parse_body(&request.body())?, load_user()?),

        (DELETE) [/uniforms/(id: i32)] =>
            |id| delete_uniform(id, load_user()?),

        // todos
        (GET) [/todos] =>
            || get_todos(load_user()?),

        (POST) [/todos] =>
            || add_todo_for_members(parse_body(&request.body())?, load_user()?),

        (POST) [/todos/(id: i32)] =>
            |id| mark_todo_as_complete(id, load_user()?),

        // songs
        (GET) [/repertoire/(id: i32)?(details: bool)] =>
            |id, details| get_song(id, details, load_user()?),

        (GET) [/repertoire] =>
            || get_songs(load_user()?),

        (GET) [/public_songs] =>
            || get_public_songs(),

        (POST) [/repertoire] =>
            || new_song(parse_body(&request.body())?, load_user()?),

        (POST) [/repertoire/(id: i32)] =>
            |id| update_song(id, parse_body(&request.body())?, load_user()?),

        (POST) [/repertoire/(id: i32)/current] =>
            |id| set_song_as_current(id, load_user()?),

        (POST) [/repertoire/(id: i32)/not_current] =>
            |id| set_song_as_not_current(id, load_user()?),

        (DELETE) [/repertoire/(id: i32)] =>
            |id| delete_song(id, load_user()?),

        // song links
        (POST) [/repertoire/(id: i32)/links] =>
            |id| new_song_link(id, parse_body(&request.body())?, load_user()?),

        (GET) [/repertoire/links/(id: i32)] =>
            |id| get_song_link(id, load_user()?),

        (DELETE) [/repertoire/links/(id: i32)] =>
            |id| remove_song_link(id, load_user()?),

        (POST) [/repertoire/links/(id: i32)] =>
            |id| update_song_link(id, parse_body(&request.body())?, load_user()?),

        (GET) [/repertoire/cleanup_files?(confirm: bool)] =>
            |confirm| cleanup_song_files(confirm, load_user()?),

        // semesters
        (GET) [/semesters] =>
            || get_semesters(load_user()?),

        (GET) [/semesters/current] =>
            || get_current_semester(),

            (GET) [/semesters/(name: String)] =>
            |name| get_semester(name, load_user()?),

        (POST) [/semesters] =>
            || new_semester(parse_body(&request.body())?, load_user()?),

        (POST) [/semesters/(name: String)] =>
            |name| edit_semester(name, parse_body(&request.body())?, load_user()?),

        (POST) [/semesters/(name: String)/set_current] =>
            |name| set_current_semester(name, load_user()?),

        (DELETE) [/semesters/(name: String)?(confirm: bool)] =>
            |name, confirm| delete_semester(name, confirm, load_user()?),

        // permissions and roles
        (GET) [/role_permissions] =>
            || get_current_role_permissions(load_user()?),

        (GET) [/member_roles] =>
            || get_current_officers(load_user()?),

        (GET) [/permissions/(member: String)] =>
            |member| member_permissions(member, load_user()?),

        (POST) [/permissions/(position: String)/enable] =>
            |position| add_permission_for_role(position, parse_body(&request.body())?, load_user()?),

        (POST) [/permissions/(position: String)/disable] =>
            |position| remove_permission_for_role(position, parse_body(&request.body())?, load_user()?),

        (POST) [/roles/add] =>
            || add_officership(parse_body(&request.body())?, load_user()?),

        (POST) [/roles/remove] =>
            || remove_officership(parse_body(&request.body())?, load_user()?),

        // fees and transactions
        (GET) [/fees] =>
            || get_fees(load_user()?),

        (POST) [/fees/(name: String)/(new_amount: i32)] =>
            |name, amount| update_fee_amount(name, amount, load_user()?),

        (POST) [/fees/charge_dues] =>
            || charge_dues(load_user()?),

        (POST) [/fees/charge_late_dues] =>
            || charge_late_dues(load_user()?),

        (POST) [/fees/create_batch] =>
            || batch_transactions(parse_body(&request.body())?, load_user()?),

        (GET) [/transactions] =>
            || get_transactions(load_user()?),

        (GET) [/transactions/(member: String)] =>
            |member| get_member_transactions(member, load_user()?),

        (POST) [/transactions] =>
            || add_transactions(parse_body(&request.body())?, load_user()?),

        (POST) [/transactions/(id: i32)/resolve/(resolved: bool)] =>
            |id, resolved| resolve_transaction(id, resolved, load_user()?),

        // static data
        (GET) [/static] =>
            || static_data(),

        (GET) [/media_types] =>
            || get_media_types(load_user()?),

        (GET) [/permissions] =>
            || get_permissions(load_user()?),

        (GET) [/roles] =>
            || get_roles(load_user()?),

        (GET) [/event_types] =>
            || get_event_types(load_user()?),

        (GET) [/section_types] =>
            || get_section_types(load_user()?),

        (GET) [/transaction_types] =>
            || get_transaction_types(load_user()?),

        (POST) [/upload_frontend] =>
            || {
                load_user()?;
                crate::util::write_zip_to_directory(request.body(), "../httpsdocs/glubhub/")
                    .map(|_| basic_success())
            },

        (POST) [/send_emails] =>
            || send_emails(parse_body(&request.body())?, load_user()?),
    )
}

#[derive(Deserialize)]
struct Since {
    pub timestamp: i64,
}

fn send_emails(since: Since, _user: User) -> GreaseResult<Value> {
    let since_time = Local
        .from_utc_datetime(&NaiveDateTime::from_timestamp(since.timestamp / 1000, 0))
        .naive_local();

    crate::cron::send_event_emails(Some(since_time))?;

    Ok(basic_success())
}

/// Returns a basic success message.
///
/// Returns the following with a 200 status code:
/// ```json
/// {
///     "message": "success!"
/// }
/// ```
pub fn basic_success() -> Value {
    json!({ "message": "success!" })
}

/// Returns an id for new resources.
///
/// Returns the following with a 200 status code:
/// ```json
/// {
///     "id": int
/// }
/// ```
pub fn id_json(id: i32) -> Value {
    json!({ "id": id })
}
