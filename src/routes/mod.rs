//! All routes for the API.
//!
//! Check the root of the crate for the full API layout.

pub mod event_routes;
pub mod member_routes;
pub mod misc_routes;
pub mod officer_routes;
pub mod repertoire_routes;

use cgi::http::response;
use serde_json::{json, Value};
use std::panic::{self, AssertUnwindSafe};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

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
pub fn handle_request(mut request: cgi::Request) -> cgi::Response {
    let mut response = None;

    let result = {
        panic::catch_unwind(AssertUnwindSafe(|| {
            build_request(request).reply(router_filter())
        }))
    };

    match result {
        Ok(()) => response.unwrap(),
        Err(error) => crate::util::log_panic(&request, format!("{:?}", error)),
    }
}

pub fn build_request(mut cgi_request: cgi::Request) -> warp::test::RequestBuilder {
    let uri = {
        let path = cgi_request
            .headers()
            .get("x-cgi-path-info")
            .map(|uri| uri.to_str().unwrap())
            .unwrap_or("/");
        let param_str = cgi_request
            .headers()
            .get("x-cgi-query-string")
            .map(|uri| uri.to_str().unwrap())
            .unwrap_or("");
        format!(
            "https://gleeclub.gatech.edu{}?{}",
            utf8_percent_encode(&path, DEFAULT_ENCODE_SET).to_string(),
            utf8_percent_encode(&param_str, DEFAULT_ENCODE_SET).to_string()
        )
    };

    let mut new_request = warp::test::request()
        .method(cgi_request.method().as_ref())
        .path(&uri)
        .body(cgi_request.body());
    for (header, value) in cgi_request.headers().iter() {
        new_request = new_request.header(header, value);
    }

    new_request
}

pub fn options_response() -> impl warp::Reply {
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

pub fn router_filter() -> impl warp::Filter {
    warp::options().map(options_response)
        .or(route_filters::authorization())
        .or(route_filters::repertoire())
        .or(route_filters::events())
        .or(route_filters::absence_requests())
        .or(route_filters::gig_requests())
        .or(route_filters::variables())
        .or(route_filters::announcements())
}

mod route_filters {
    use std::collections::HashMap;
    use warp::filters::method::v2::*;
    use warp::filters::query::query;
    use warp::body::json;
    use warp::filters::header;
    use warp::path;
    use db::*;
    use auth::User;

    pub fn authorization() -> impl warp::Filter {
        use crate::routes::member_routes::{login, logout};

        let login_route = path!("login")
            .and(post())
            .and(json::<LoginInfo>())
            .and_then(login);
        let logout_route = path!("logout")
            .and(get())
            .and(User::filter())
            .and_then(logout);

        login_route.or(logout_route)
    }

    pub fn members() -> impl warp::Filter {
        use crate::routes::member_routes::*;

            let new_member_route = post().and(json()).and_then(new_member);
            let user_route = path!("user")
            .and(get())
            .and(header::optional("token"))
            .and_then(get_current_user);

        let new_member_route = path!("member").and(json()).and_then(new_member);

        let member_routes = path!("member").and(User::filter()).and({


        })

        //         (GET)    [/user] => get_current_user,
//         (GET)    [/members/(email: String)?(grades: Option<bool>)?(details: Option<bool>)] => get_member,
//         (GET)    [/members/(email: String)/attendance] => get_member_attendance_for_semester,
//         (GET)    [/members?(grades: Option<bool>)?(include: Option<String>)] => get_members,
//         (POST)   [/members] => new_member,
//         (POST)   [/members/confirm] => confirm_for_semester,
//         (POST)   [/members/(email: String)/(semester: String)] => update_member_semester,
//         (DELETE) [/members/(email: String)/(semester: String)] => mark_member_inactive_for_semester,
//         (POST)   [/members] => new_member,
//         (POST)   [/members/profile] => update_member_profile,
//         (POST)   [/members/(email: String)] => update_member_as_officer,
//         (POST)   [/members/(email: String)/login_as] => login_as_member,
//         (DELETE) [/members/(email: String)?(confirm: Option<bool>)] => delete_member,

    }

    pub fn repertoire() -> impl warp::Filter {
        use crate::routes::repertoire_routes::*;

        path!("repertoire").and(User::filter()).and({
            let get_songs_route = get().and_then(get_songs);
            let new_song_route = post().and(json()).and_then(new_song);

            let routes_for_single_song = path!(i32).and({
                let get_song_route = get().and(query::<HashMap<String, bool>>()).and_then(get_song);
                let update_song_route = post().and(json()).and_then(update_song);
                let set_current_route = post().and(path!("current")).and_then(set_song_as_current);
                let set_not_current_route = post().and(path!("not_current")).and_then(set_song_as_not_current);
                let delete_song_route = delete().and_then(delete_song);

                get_song_route
                    .or(update_song_route)
                    .or(set_current_route)
                    .or(set_not_current_route)
                    .or(delete_song_route)
            });

            routes_for_single_song
                .and(get_songs_route)
                .and(new_song_route)
        })
    }

    pub fn events() -> impl warp::Filter {
        use crate::routes::event_routes::*;

        path!("events").and(User::filter()).and({
            let new_event_route = post().and(json()).and_then(new_event);
            let all_events_route = get().and(query::<HashMap<String, String>>()).and_then(get_events);

            let routes_for_specific_song = path!(i32).and({
                let get_event_route = get().and(query::<HashMap<String, bool>>()).and_then(get_event);
                let update_event_route = post().and(json()).and_then(update_event);
                let delete_event_route = delete().and_then(delete_event);

                get_event_route.or(update_event_route).or(delete_event_route)
            });

            new_event_route.or(all_events_route).or(routes_for_specific_song)
        })
    }

    pub fn absence_requests() -> impl warp::Filter {
        use crate::routes::event_routes::*;

        path!("absence_requests").and(User::filter()).and({
            let get_all_requests = get().and_then(get_absence_requests);

            let routes_for_specific_request = path!(i32).and({
                let get_request_route = get().and_then(get_absence_request);
                let is_excused_route = get().and(path!("is_excused")).and_then(member_is_excused);
                let submit_request_route = post().and(json()).and_then(submit_absence_request);

                let respond_to_request_routes = path!(String).and({
                    let approve_route = path!("approve").and_then(approve_absence_request);
                    let deny_route = path!("deny").and_then(deny_absence_request);

                    approve_route.or(deny_route)
                });

                get_request_route
                    .or(is_excused_route)
                    .or(submit_request_route)
                    .or(respond_to_request_routes)
            });

            get_all_requests.or(routes_for_specific_request)
        })
    }

    pub fn gig_requests() -> impl warp::Filter {
        use crate::routes::event_routes::*;

        path!("gig_requests").and(User::filter()).and({
            let new_request_route = post().and(json()).and_then(new_gig_request);
            let all_requests_route = get().and(query::<HashMap<String, bool>>()).and_then(get_gig_requests);

            let routes_for_specific_request = path!(i32).and({
                let get_request_route = get().and_then(get_gig_request);
                let dismiss_request_route = post().and(path!("dismiss")).and_then(dismiss_gig_request);
                let reopen_request_route = post().and(path!("reopen")).and_then(reopen_gig_request);
                let accept_request_route = post().and(path!("create_event")).and(json()).and_then(create_event_from_gig_request);

                get_request_route.or(dismiss_request_route).or(reopen_request_route).or(accept_request_route)
            });

            new_request_route.or(all_requests_route).or(routes_for_specific_request)
        })

    }

    pub fn variables() -> impl warp::Filter {
        use crate::routes::misc_routes::*;

        path!("variables" / String).and(User::filter()).and({
            let get_route = get().and_then(get_variable);
            let set_route = post().and(json()).and_then(set_variable);
            let unset_route = delete().and_then(unset_variable);

            get_route.or(set_route).or(unset_route)
        })
    }

    pub fn announcements() -> impl warp::Filter {
        use crate::routes::officer_routes::*;

        path!("announcements").and(User::filter()).and({
            let all_announcements_route = get().and(query::<HashMap<String, bool>>()).and_then(get_announcements);
            let new_announcement_route = post().and(json()).and_then(make_new_announcement);

            let routes_for_specific_announcement = path!(i32).and({
                let get_route = get().and_then(get_announcement);
                let archive_route = post().and(path!("archive")).and_then(archive_announcement);

                get_route.or(archive_route)
            });

            all_announcements_route.or(new_announcement_route).or(routes_for_specific_announcement)
        })

    }

    pub fn google_docs() -> impl warp::Filter {
        use crate::routes::officer_routes::*;

        path!("google_docs").and(User::filter()).and({
            let all_docs_route = get().and_then(get_google_docs);
            let new_docs_route = post().and(json()).and_then(new_google_doc);

            let routes_for_specific_docs = path!(String).and({
                let get_route = get().and_then(get_google_doc);
                let modify_route = post().and(json()).and_then(modify_google_doc);
                let delete_route = delete().and_then(delete_google_doc);

                get_route.or(modify_route).or(delete_route)
            });

            all_docs_route.or(new_docs_route).or(routes_for_specific_docs)
        })


    }
}


/// Handles routing of incoming requests.
///
/// See the root of the crate for the API layout and
/// [router](router/macro.router.html) for the way this function works.
// pub fn handle(request: &cgi::Request) -> GreaseResult<Value> {
//     router!(request,

//         // authorization
//         (POST)   [/login]  => login,
//         (GET)    [/logout] => logout,

//         // members
//         (GET)    [/user] => get_current_user,
//         (GET)    [/members/(email: String)?(grades: Option<bool>)?(details: Option<bool>)] => get_member,
//         (GET)    [/members/(email: String)/attendance] => get_member_attendance_for_semester,
//         (GET)    [/members?(grades: Option<bool>)?(include: Option<String>)] => get_members,
//         (POST)   [/members] => new_member,
//         (POST)   [/members/confirm] => confirm_for_semester,
//         (POST)   [/members/(email: String)/(semester: String)] => update_member_semester,
//         (DELETE) [/members/(email: String)/(semester: String)] => mark_member_inactive_for_semester,
//         (POST)   [/members] => new_member,
//         (POST)   [/members/profile] => update_member_profile,
//         (POST)   [/members/(email: String)] => update_member_as_officer,
//         (POST)   [/members/(email: String)/login_as] => login_as_member,
//         (DELETE) [/members/(email: String)?(confirm: Option<bool>)] => delete_member,

//         // events
//         (GET)    [/events/(id: i32)?(full: Option<bool>)] => get_event,
//         (GET)    [/events?(full: Option<bool>)?(event_types: Option<String>)] => get_events,
//         (POST)   [/events] => new_event,
//         (POST)   [/events/(id: i32)] => update_event,
//         (DELETE) [/events/(id: i32)] => delete_event,

//         // event details
//         (GET)    [/events/(id: i32)/attendance] => get_attendance,
//         (GET)    [/events/(id: i32)/see_whos_attending] => see_whos_attending,
//         (GET)    [/events/(id: i32)/attendance/(member: String)] => get_member_attendance,
//         (POST)   [/events/(id: i32)/attendance/(member: String)] => update_attendance,
//         (POST)   [/events/(id: i32)/rsvp/(attending: bool)] => rsvp_for_event,
//         (POST)   [/events/(id: i32)/attendance/excuse_unconfirmed] => excuse_unconfirmed_for_event,
//         (GET)    [/events/(id: i32)/carpools] => get_carpools,
//         (POST)   [/events/(id: i32)/carpools] => update_carpools,
//         (GET)    [/events/(id: i32)/setlist] => get_setlist,
//         (POST)   [/events/(id: i32)/setlist] => edit_setlist,

//         // absence requests
//         (GET)    [/absence_requests] => get_absence_requests,
//         (GET)    [/absence_requests/(event_id: i32)] => get_absence_request,
//         (GET)    [/absence_requests/(event_id: i32)/is_excused] => member_is_excused,
//         (POST)   [/absence_requests/(event_id: i32)/(member: String)/approve] => approve_absence_request,
//         (POST)   [/absence_requests/(event_id: i32)/(member: String)/deny] => deny_absence_request,
//         (POST)   [/absence_requests/(event_id: i32)] => submit_absence_request,

//         // gig requests
//         (GET)    [/gig_requests/(id: i32)] => get_gig_request,
//         (GET)    [/gig_requests?(all: Option<bool>)] => get_gig_requests,
//         (POST)   [/gig_requests] => new_gig_request,
//         (POST)   [/gig_requests/(id: i32)/dismiss] => dismiss_gig_request,
//         (POST)   [/gig_requests/(id: i32)/reopen] => reopen_gig_request,
//         (POST)   [/gig_requests/(id: i32)/create_event] => create_event_from_gig_request,

//         // variables
//         (GET)    [/variables/(key: String)] => get_variable,
//         (POST)   [/variables/(key: String)] => set_variable,
//         (DELETE) [/variables/(key: String)] => unset_variable,

//         // announcements
//         (GET)    [/announcements/(id: i32)] => get_announcement,
//         (GET)    [/announcements?(all: Option<bool>)] => get_announcements,
//         (POST)   [/announcements] => make_new_announcement,
//         (POST)   [/announcements/(id: i32)/archive] => archive_announcement,

//         // google docs
//         (GET)    [/google_docs/(name: String)] => get_google_doc,
//         (GET)    [/google_docs] => get_google_docs,
//         (POST)   [/google_docs] => new_google_doc,
//         (POST)   [/google_docs/(name: String)] => modify_google_doc,
//         (DELETE) [/google_docs/(name: String)] => delete_google_doc,

//         // meeting minutes
//         (GET)    [/meeting_minutes/(id: i32)] => get_meeting_minutes,
//         (GET)    [/meeting_minutes] => get_all_meeting_minutes,
//         (POST)   [/meeting_minutes] => new_meeting_minutes,
//         (POST)   [/meeting_minutes/(id: i32)] => modify_meeting_minutes,
//         (GET)    [/meeting_minutes/(id: i32)/email] => send_minutes_as_email,
//         (DELETE) [/meeting_minutes/(id: i32)] => delete_meeting_minutes,

//         // uniforms
//         (GET)    [/uniforms/(id: i32)] => get_uniform,
//         (GET)    [/uniforms] => get_uniforms,
//         (POST)   [/uniforms] => new_uniform,
//         (POST)   [/uniforms/(id: i32)] => modify_uniform,
//         (DELETE) [/uniforms/(id: i32)] => delete_uniform,

//         // todos
//         (GET)    [/todos] => get_todos,
//         (POST)   [/todos] => add_todo_for_members,
//         (POST)   [/todos/(id: i32)] => mark_todo_as_complete,

//         // songs
//         (GET)    [/repertoire/(id: i32)?(details: Option<bool>)] => get_song,
//         (GET)    [/repertoire] => get_songs,
//         (POST)   [/repertoire] => new_song,
//         (POST)   [/repertoire/(id: i32)] => update_song,
//         (POST)   [/repertoire/(id: i32)/current] => set_song_as_current,
//         (POST)   [/repertoire/(id: i32)/not_current] => set_song_as_not_current,
//         (DELETE) [/repertoire/(id: i32)] => delete_song,

//         // song links
//         (POST)   [/repertoire/(id: i32)/links] => new_song_link,
//         (GET)    [/repertoire/links/(id: i32)] => get_song_link,
//         (DELETE) [/repertoire/links/(id: i32)] => remove_song_link,
//         (POST)   [/repertoire/links/(id: i32)] => update_song_link,
//         (POST)   [/repertoire/upload] => upload_file,
//         (GET)    [/repertoire/cleanup_files?(confirm: Option<bool>)] => cleanup_song_files,

//         // semesters
//         (GET)    [/semesters] => get_semesters,
//         (GET)    [/semesters/current] => get_current_semester,
//         (GET)    [/semesters/(name: String)] => get_semester,
//         (POST)   [/semesters] => new_semester,
//         (POST)   [/semesters/(name: String)] => edit_semester,
//         (POST)   [/semesters/(name: String)/set_current] => set_current_semester,
//         (DELETE) [/semesters/(name: String)?(confirm: Option<bool>)] => delete_semester,

//         // permissions and roles
//         (GET)    [/role_permissions] => get_current_role_permissions,
//         (GET)    [/member_roles] => get_current_officers,
//         (GET)    [/permissions/(member: String)] => member_permissions,
//         (POST)   [/permissions/(position: String)/enable] => add_permission_for_role,
//         (POST)   [/permissions/(position: String)/disable] => remove_permission_for_role,
//         (POST)   [/roles/add] => add_officership,
//         (POST)   [/roles/remove] => remove_officership,

//         // fees and transactions
//         (GET)    [/fees] => get_fees,
//         (POST)   [/fees/(name: String)/(new_amount: i32)] => update_fee_amount,
//         (POST)   [/fees/(name: String)/apply] => apply_fee_for_all_active_members,
//         (GET)    [/transactions/(member: String)] => get_member_transactions,
//         (POST)   [/transactions] => add_transactions,

//         // static data
//         (GET)    [/static] => static_data,
//         (GET)    [/media_types] => get_media_types,
//         (GET)    [/permissions] => get_permissions,
//         (GET)    [/roles] => get_roles,
//         (GET)    [/event_types] => get_event_types,
//         (GET)    [/section_types] => get_section_types,
//         (GET)    [/transaction_types] => get_transaction_types,
//     )
// }

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
