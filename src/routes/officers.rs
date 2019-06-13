use app_route::AppRoute;
use super::{OptionalIdQuery, OptionalEmailQuery};
use crate::check_for_permission;
use db::models::*;
use auth::User;
use error::{GreaseError, GreaseResult};
use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use extract_derive::Extract;

#[derive(AppRoute, Debug)]
#[route("/announcements")]
pub struct AnnouncementsRequest {
    #[query]
    pub query: OptionalIdQuery,
}

pub fn get_announcements(req: AnnouncementsRequest, user: User) -> GreaseResult<Value> {
    if let Some(announcement_id) = req.query.id {
        Announcement::load(announcement_id, &user.conn).map(|announcement| json!(announcement))
    } else {
        Announcement::load_all_for_semester(&user.member.active_semester.semester, &user.conn)
            .map(|announcements| json!(announcements))
    }
}

#[derive(AppRoute, Debug)]
#[route("/announcements")]
pub struct NewAnnouncementRequest {}

#[derive(Deserialize, Extract)]
pub struct NewAnnouncement {
    pub content: String,
}

pub fn make_new_announcement(_req: NewAnnouncementRequest, (user, new_announcement): (User, NewAnnouncement)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-announcements");
    Announcement::insert(&new_announcement.content, &user.member.member.email, &user.member.active_semester.semester, &user.conn)
        .map(|new_id| json!({ "id": new_id }))
}

#[derive(AppRoute, Debug)]
#[route("/google_docs")]
pub struct GoogleDocsRequest {
    #[query]
    pub query: GoogleDocsQuery,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleDocsQuery {
    pub name: Option<String>,
}

pub fn get_google_docs(req: GoogleDocsRequest, user: User) -> GreaseResult<Value> {
    if let Some(doc_name) = req.query.name {
        GoogleDoc::load(&doc_name, &user.conn).map(|doc| json!(doc))
    } else {
        GoogleDoc::load_all(&user.conn).map(|docs| json!(docs))
    }
}

pub fn modify_google_docs(req: GoogleDocsRequest, (user, changed_doc): (User, GoogleDoc)) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-links");
    if let Some(doc_name) = req.query.name {
        GoogleDoc::update(&doc_name, &changed_doc, &user.conn).map(|_| json!({
            "message": "success!"
        }))
    } else {
        GoogleDoc::insert(&changed_doc, &user.conn).map(|_| json!({
            "message": "success!"
        }))
    }
}

#[derive(AppRoute, Debug)]
#[route("/meeting_minutes")]
pub struct MeetingMinutesRequest {
    #[query]
    pub query: OptionalIdQuery,
}

// Fee
// MemberRole
// MeetingMinutes
// Permission
// Role
// RolePermission
// SectionType
// Semester
// Todo
// Transaction
// TransactionType
// Uniform
