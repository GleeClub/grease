use async_graphql::{Context, Guard, Result};

use crate::db::DbConn;
use crate::models::member::Member;
use crate::models::permissions::MemberPermission;

pub struct LoggedIn;

#[async_trait::async_trait]
impl Guard for LoggedIn {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        if ctx.data_opt::<Member>().is_some() {
            Ok(())
        } else {
            Err("User must be logged in".into())
        }
    }
}

pub struct Permission {
    name: &'static str,
    event_type: Option<String>,
}

impl Permission {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            event_type: None,
        }
    }

    pub fn for_type(self, event_type: impl Into<String>) -> Self {
        Self {
            name: self.name,
            event_type: Some(event_type.into()),
        }
    }

    pub async fn granted_to(&self, member: &str, conn: &DbConn) -> Result<bool> {
        let permissions = MemberPermission::for_member(member, conn).await?;

        Ok(permissions.iter().any(|permission| {
            permission.name == self.name
                && (permission.event_type.is_none() || permission.event_type == self.event_type)
        }))
    }

    pub async fn ensure_granted_to(&self, member: &str, conn: &DbConn) -> Result<()> {
        if self.granted_to(member, conn).await? {
            Ok(())
        } else {
            Err(format!("Permission {} required", self.name).into())
        }
    }

    pub const PROCESS_GIG_REQUESTS: Self = Self::new("process-gig-requests");
    pub const PROCESS_ABSENCE_REQUESTS: Self = Self::new("process-absence-requests");

    pub const EDIT_REPERTOIRE: Self = Self::new("edit-repertoire");

    pub const EDIT_LINKS: Self = Self::new("edit-links");
    pub const EDIT_UNIFORMS: Self = Self::new("edit-uniforms");

    pub const EDIT_SEMESTER: Self = Self::new("edit-semester");
    pub const EDIT_PERMISSIONS: Self = Self::new("edit-permissions");
    pub const EDIT_OFFICERS: Self = Self::new("edit-officers");

    pub const VIEW_TRANSACTIONS: Self = Self::new("view-transactions");
    pub const EDIT_TRANSACTION: Self = Self::new("edit-transaction");

    pub const ADD_MULTI_TODO: Self = Self::new("add-multi-todo");

    pub const EDIT_MINUTES: Self = Self::new("edit-minutes");
    pub const VIEW_COMPLETE_MINUTES: Self = Self::new("view-complete-minutes");

    pub const EDIT_USER: Self = Self::new("edit-user");
    pub const SWITCH_USER: Self = Self::new("switch-user");
    pub const DELETE_USER: Self = Self::new("delete-user");
    pub const VIEW_USERS: Self = Self::new("view-users");
    pub const VIEW_USER_PRIVATE_DETAILS: Self = Self::new("view-user-private-details");

    pub const CREATE_EVENT: Self = Self::new("create-event");
    pub const MODIFY_EVENT: Self = Self::new("modify-event");
    pub const EDIT_ALL_EVENTS: Self = Self::new("edit-all-events");
    pub const DELETE_EVENT: Self = Self::new("delete-event");

    pub const EDIT_ATTENDANCE: Self = Self::new("edit-attendance");
    pub const EDIT_ATTENDANCE_OWN_SECTION: Self = Self::new("edit-attendance-own-section");

    pub const EDIT_CARPOOLS: Self = Self::new("edit-carpool");
}

#[async_trait::async_trait]
impl Guard for Permission {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(user) = ctx.data_opt::<Member>() {
            let conn = DbConn::from_ctx(ctx);
            if self.granted_to(&user.email, &conn).await? {
                return Ok(());
            }
        }

        Err(format!("Permission {} required", self.name).into())
    }
}
