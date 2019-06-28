//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website.
//!
//! The backend is a JSON API, agnostic of the platform of interaction. This
//! leaves the site and its data to be available on other platforms in the future,
//! including potentially a phone app or a CLI.
//!
//! ## Basic JSON information:
//!
//! The types of data returned as fields are notated as such:
//!   * **string**: A string of characters
//!   * **integer**: An integer number
//!   * **float**: A floating-point number
//!   * **boolean**: "true" or "false"
//!   * **datetime**: An RFC 3339 datetime, formatted as a string
//!   * **date**: An RFC 3339 date, formatted as a string
//!   * **?**: Any field followed by a question mark can be null, otherwise it is never null
//!
//! ## All Routes:
//!
//! Below is the layout of the API. Click on the handler of any endpoint to find
//! out more about its usage.
//!
//! All routes have the following constraints:
//!   * Path Parameters: A list of parameters, their types, whether they are required, and
//!       a short description. If this section is missing, none are expected.
//!   * Query Parameters: The same as path paramters. Assume none are accepted if
//!       this section is missing.
//!   * Required Permissions: Describes what permissions are required to use the endpoint.
//!       If this section is missiong, anyone can use the endpoint.
//!   * Input Format: The expected format of all input data for POST requests. If this
//!       section is missing, POST data will be ignored.
//!   * Return Format: The format of data to be returned. If this section is missing,
//!       a [basic success](routes/fn.basic_success.html) is always returned by default
//!       on success.
//!
//! ### Authorization:
//!
//!   Method   | Route   | Handler
//! -----------|---------|------------------------------------------------
//! **POST**   | /login  | [login](./routes/member_routes/fn.login.html)
//! **GET**    | /logout | [logout](./routes/member_routes/fn.logout.html)
//!
//! ### Members:
//!
//!   Method   | Route                           | Handler
//! -----------|---------------------------------|-------------------------------------------------------------------------------------------------------
//! **GET**    | /members/{*email*}              | [get_member](./routes/member_routes/fn.get_member.html)
//! **GET**    | /members/{*email*}/attendance   | [get_member_attendance_for_semester](./routes/event_routes/fn.get_member_attendance_for_semester.html)
//! **GET**    | /members                        | [get_members](./routes/member_routes/fn.get_members.html)
//! **POST**   | /members                        | [new_member](./routes/member_routes/fn.new_member.html)
//! **POST**   | /members/register               | [register_for_semester](./routes/member_routes/fn.register_for_semester.html)
//! **POST**   | /members/{*email*}/{*semester*} | [update_member_semester](./routes/member_routes/fn.update_member_semester.html)
//! **DELETE** | /members/{*email*}/{*semester*} | [mark_member_inactive_for_semester](./routes/member_routes/fn.mark_member_inactive_for_semester.html)
//! **POST**   | /members                        | [new_member](./routes/member_routes/fn.new_member.html)
//! **POST**   | /members/profile                | [update_member_profile](./routes/member_routes/fn.update_member_profile.html)
//! **POST**   | /members/{*email*}              | [update_member_as_officer](./routes/member_routes/fn.update_member_as_officer.html)
//! **POST**   | /members/{*email*}/login_as     | [login_as_member](./routes/member_routes/fn.login_as_member.html)
//! **DELETE** | /members/{*email*}              | [delete_member](./routes/member_routes/fn.delete_member.html)
//!
//! ### Events:
//!
//!   Method   | Route          | Handler
//! -----------|----------------|-------------
//! **GET**    | /events/{*id*} | [get_event](./routes/event_routes/fn.get_event.html)
//! **GET**    | /events        | [get_events](./routes/event_routes/fn.get_events.html)
//! **POST**   | /events        | [new_event](./routes/event_routes/fn.new_event.html)
//! **POST**   | /events/{*id*} | [update_event](./routes/event_routes/fn.update_event.html)
//! **DELETE** | /events/{*id*} | [delete_event](./routes/event_routes/fn.delete_event.html)
//!
//! ### Event Details:
//!
//!   Method   | Route                                        | Handler
//! -----------|----------------------------------------------|-----------------------------------------------------------------------------------------
//! **GET**    | /events/{*id*}/attendance                    | [get_attendance](routes/event_routes/fn.get_attendance.html)
//! **GET**    | /events/{*id*}/attendance/{*member*}         | [get_member_attendance](routes/event_routes/fn.get_member_attendance.html)
//! **POST**   | /events/{*id*}/attendance/{*member*}         | [update_attendance](routes/event_routes/fn.update_attendance.html)
//! **POST**   | /events/{*id*}/attendance/excuse_unconfirmed | [excuse_unconfirmed_for_event](routes/event_routes/fn.excuse_unconfirmed_for_event.html)
//! **GET**    | /events/{*id*}/carpools                      | [get_carpools](routes/event_routes/fn.get_carpools.html)
//! **POST**   | /events/{*id*}/carpools                      | [update_carpools](routes/event_routes/fn.update_carpools.html)
//! **GET**    | /events/{*id*}/setlist                       | [get_setlist](routes/event_routes/fn.get_setlist.html)
//! **POST**   | /events/{*id*}/setlist                       | [edit_setlist](routes/event_routes/fn.edit_setlist.html)
//!
//! ### Absence Requests:
//!
//!   Method   | Route                                               | Handler
//! -----------|-----------------------------------------------------|-------------------------------------------------------------------------------
//! **GET**    | /absence_requests                                   | [get_absence_requests](routes/event_routes/fn.get_absence_requests.html)
//! **GET**    | /absence_requests/{*eventId*}/{*member*}            | [get_absence_request](routes/event_routes/fn.get_absence_request.html)
//! **GET**    | /absence_requests/{*eventId*}/{*member*}/is_excused | [member_is_excused](routes/event_routes/fn.member_is_excused.html)
//! **POST**   | /absence_requests/{*eventId*}/{*member*}/approve    | [approve_absence_request](routes/event_routes/fn.approve_absence_request.html)
//! **POST**   | /absence_requests/{*eventId*}/{*member*}/deny       | [deny_absence_request](routes/event_routes/fn.deny_absence_request.html)
//! **POST**   | /absence_requests/{*eventId*}                       | [submit_absence_request](routes/event_routes/fn.submit_absence_request.html)
//!
//! ### Gig Requests:
//!
//!   Method   | Route                             | Handler
//! -----------|-----------------------------------|-------------------------------------------------------------------------------------------
//! **GET**    | /gig_requests/{*id*}              | [get_gig_request](routes/event_routes/fn.get_gig_request.html)
//! **GET**    | /gig_requests                     | [get_gig_requests](routes/event_routes/fn.get_gig_requests.html)
//! **POST**   | /gig_requests                     | [new_gig_request](routes/event_routes/fn.new_gig_request.html)
//! **POST**   | /gig_requests/{*id*}/dismiss      | [dismiss_gig_request](routes/event_routes/fn.dismiss_gig_request.html)
//! **POST**   | /gig_requests/{*id*}/reopen       | [reopen_gig_request](routes/event_routes/fn.reopen_gig_request.html)
//! **POST**   | /gig_requests/{*id*}/create_event | [create_event_from_gig_request](routes/event_routes/fn.create_event_from_gig_request.html)
//!
//! ### Variables:
//!
//!   Method   | Route              | Handler
//! -----------|--------------------|------------------------------------------------------------
//! **GET**    | /variables/{*key*} | [get_variable](routes/misc_routes/fn.get_variable.html)
//! **POST**   | /variables/{*key*} | [set_variable](routes/misc_routes/fn.set_variable.html)
//! **DELETE** | /variables/{*key*} | [unset_variable](routes/misc_routes/fn.unset_variable.html)
//!
//! ### Announcements:
//!
//!   Method   | Route                 | Handler
//! -----------|-----------------------|------------------------------------------------------------------------------
//! **GET**    | /announcements/{*id*} | [get_announcement](routes/officer_routes/fn.get_announcement.html)
//! **GET**    | /announcements        | [get_announcements](routes/officer_routes/fn.get_announcements.html)
//! **POST**   | /announcements        | [make_new_announcement](routes/officer_routes/fn.make_new_announcement.html)
//! **POST**   | /announcements/{*id*} | [archive_announcement](routes/officer_routes/fn.archive_announcement.html)
//!
//! ### Google Docs:
//!
//!   Method   | Route                 | Handler
//! -----------|-----------------------|----------------------------------------------------------------------
//! **GET**    | /google_docs/{*name*} | [get_google_doc](routes/officer_routes/fn.get_google_doc.html)
//! **GET**    | /google_docs]         | [get_google_docs](routes/officer_routes/fn.get_google_docs.html)
//! **POST**   | /google_docs]         | [new_google_doc](routes/officer_routes/fn.new_google_doc.html)
//! **POST**   | /google_docs/{*name*} | [modify_google_doc](routes/officer_routes/fn.modify_google_doc.html)
//! **DELETE** | /google_docs/{*name*} | [delete_google_doc](routes/officer_routes/fn.delete_google_doc.html)
//!
//! ### Meeting Minutes:
//!
//!   Method   | Route                         | Handler
//! -----------|-------------------------------|---------------------------------------------------------------------------------
//! **GET**    | /meeting_minutes/{*id*}       | [get_meeting_minutes](routes/officer_routes/fn.get_meeting_minutes.html)
//! **GET**    | /meeting_minutes              | [get_all_meeting_minutes](routes/officer_routes/fn.get_all_meeting_minutes.html)
//! **POST**   | /meeting_minutes              | [new_meeting_minutes](routes/officer_routes/fn.new_meeting_minutes.html)
//! **POST**   | /meeting_minutes/{*id*}       | [modify_meeting_minutes](routes/officer_routes/fn.modify_meeting_minutes.html)
//! **GET**    | /meeting_minutes/{*id*}/email | [send_minutes_as_email](routes/officer_routes/fn.send_minutes_as_email.html)
//! **DELETE** | /meeting_minutes/{*id*}       | [delete_meeting_minutes](routes/officer_routes/fn.delete_meeting_minutes.html)
//!
//! ### Uniforms:
//!
//!   Method   | Route            | Handler
//! -----------|------------------|---------------------------------------------------------------
//! **GET**    | /uniforms/{*id*} | [get_uniform](routes/officer_routes/fn.get_uniform.html)
//! **GET**    | /uniforms]       | [get_uniforms](routes/officer_routes/fn.get_uniforms.html)
//! **POST**   | /uniforms]       | [new_uniform](routes/officer_routes/fn.new_uniform.html)
//! **POST**   | /uniforms/{*id*} | [modify_uniform](routes/officer_routes/fn.modify_uniform.html)
//! **DELETE** | /uniforms/{*id*} | [delete_uniform](routes/officer_routes/fn.delete_uniform.html)
//!
//! ### Todos:
//!
//!   Method   | Route         | Handler
//! -----------|---------------|-----------------------------------------------------------------------------
//! **GET**    | /todos        | [get_todos](routes/officer_routes/fn.get_todos.html)
//! **POST**   | /todos        | [add_todo_for_members](routes/officer_routes/fn.add_todo_for_members.html)
//! **POST**   | /todos/{*id*} | [mark_todo_as_complete](routes/officer_routes/fn.mark_todo_as_complete.html)
//!
//! ### Songs:
//!
//!   Method   | Route                          | Handler
//! -----------|--------------------------------|------------------------------------------------------------------------------------
//! **GET**    | /repertoire/{*id*}             | [get_song](routes/repertoire_routes/fn.get_song.html)
//! **GET**    | /repertoire                    | [get_songs](routes/repertoire_routes/fn.get_songs.html)
//! **POST**   | /repertoire                    | [new_song](routes/repertoire_routes/fn.new_song.html)
//! **POST**   | /repertoire/{*id*}             | [update_song](routes/repertoire_routes/fn.update_song.html)
//! **POST**   | /repertoire/{*id*}/current     | [set_song_as_current](routes/repertoire_routes/fn.set_song_as_current.html)
//! **POST**   | /repertoire/{*id*}/not_current | [set_song_as_not_current](routes/repertoire_routes/fn.set_song_as_not_current.html)
//! **DELETE** | /repertoire/{*id*}             | [delete_song](routes/repertoire_routes/fn.delete_song.html)
//!
//! ### Song Links:
//!
//!   Method   | Route                     | Handler
//! -----------|---------------------------|--------------------------------------------------------------------------
//! **POST**   | /repertoire/{*id*}/links  | [new_song_link](routes/repertoire_routes/fn.new_song_link.html)
//! **GET**    | /repertoire/links/{*id*}  | [get_song_link](routes/repertoire_routes/fn.get_song_link.html)
//! **DELETE** | /repertoire/links/{*id*}  | [remove_song_link](routes/repertoire_routes/fn.remove_song_link.html)
//! **POST**   | /repertoire/links/{*id*}  | [update_song_link](routes/repertoire_routes/fn.update_song_link.html)
//! **POST**   | /repertoire/upload        | [upload_file](routes/repertoire_routes/fn.upload_file.html)
//! **GET**    | /repertoire/cleanup_files | [cleanup_song_files](routes/repertoire_routes/fn.cleanup_song_files.html)
//!
//! ### Semesters:
//!
//!   Method   | Route                           | Handler
//! -----------|---------------------------------|---------------------------------------------------------------------------
//! **GET**    | /semesters                      | [get_semesters](routes/officer_routes/fn.get_semesters.html)
//! **GET**    | /semester/current               | [get_current_semester](routes/officer_routes/fn.get_current_semester.html)
//! **GET**    | /semesters/{*name*}             | [get_semester](routes/officer_routes/fn.get_semester.html)
//! **POST**   | /semesters                      | [new_semester](routes/officer_routes/fn.new_semester.html)
//! **POST**   | /semesters/{*name*}             | [edit_semester](routes/officer_routes/fn.edit_semester.html)
//! **POST**   | /semesters/{*name*}/set_current | [set_current_semester](routes/officer_routes/fn.set_current_semester.html)
//! **DELETE** | /semesters/{*name*}             | [delete_semester](routes/officer_routes/fn.delete_semester.html)
//!
//! ### Permissions and Roles:
//!
//!   Method   | Route                             | Handler
//! -----------|-----------------------------------|-------------------------------------------------------------------------------------------
//! **GET**    | /role_permissions                 | [get_current_role_permissions](routes/officer_routes/fn.get_current_role_permissions.html)
//! **GET**    | /member_roles                     | [get_current_officers](routes/officer_routes/fn.get_current_officers.html)
//! **GET**    | /permissions/{*member*}           | [member_permissions](routes/officer_routes/fn.member_permissions.html)
//! **POST**   | /permissions/{*position*}/enable  | [add_permission_for_role](routes/officer_routes/fn.add_permission_for_role.html)
//! **POST**   | /permissions/{*position*}/disable | [remove_permission_for_role](routes/officer_routes/fn.remove_permission_for_role.html)
//! **POST**   | /roles/add                        | [add_officership](routes/officer_routes/fn.add_officership.html)
//! **POST**   | /roles/remove                     | [remove_officership](routes/officer_routes/fn.remove_officership.html)
//!
//! ### Fees and Transactions:
//!
//!   Method   | Route                         | Handler
//! -----------|-------------------------------|---------------------------------------------------------------------------------------------------
//! **GET**    | /fees                         | [get_fees](routes/officer_routes/fn.get_fees.html)
//! **POST**   | /fees/{*name*}/{*new_amount*} | [update_fee_amount](routes/officer_routes/fn.update_fee_amount.html)
//! **POST**   | /fees/{*name*}/apply          | [apply_fee_for_all_active_members](routes/officer_routes/fn.apply_fee_for_all_active_members.html)
//! **GET**    | /transactions/{*member*}      | [get_member_transactions](routes/officer_routes/fn.get_member_transactions.html)
//! **POST**   | /transactions                 | [add_transactions](routes/officer_routes/fn.add_transactions.html)
//!
//! ### Static Data:
//!
//!   Method   | Route              | Handler
//! -----------|--------------------|-----------------------------------------------------------------------------
//! **GET**    | /media_types       | [get_media_types](routes/repertoire_routes/fn.get_media_types.html)
//! **GET**    | /permissions       | [get_permissions](routes/officer_routes/fn.get_permissions.html)
//! **GET**    | /roles             | [get_roles](routes/officer_routes/fn.get_roles.html)
//! **GET**    | /event_types       | [get_event_types](routes/event_routes/fn.get_event_types.html)
//! **GET**    | /section_types     | [get_section_types](routes/event_routes/fn.get_section_types.html)
//! **GET**    | /transaction_types | [get_transaction_types](routes/officer_routes/fn.get_transaction_types.html)

#![feature(custom_attribute)]
#![feature(drain_filter)]
#![feature(box_syntax)]
#![feature(const_fn)]
#![recursion_limit = "128"]

extern crate base64;
extern crate cgi;
extern crate chrono;
extern crate dotenv;
extern crate glob;
extern crate grease_derive;
extern crate http;
extern crate mysql;
extern crate mysql_enum;
extern crate pinto;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
extern crate url;
// extern crate lettre;
// extern crate lettre_email;
#[cfg(test)]
extern crate mocktopus;

pub mod auth;
pub mod db;
pub mod error;
pub mod extract;
pub mod routes;
pub mod util;

use routes::handle_request;

fn main() {
    dotenv::dotenv().ok();
    cgi::handle(handle_request);
}
