use async_graphql::{Context, Object, Result};

use crate::db::DbConn;
use crate::graphql::guards::{LoggedIn, Permission};
use crate::models::event::absence_request::AbsenceRequest;
use crate::models::event::gig::GigRequest;
use crate::models::event::uniform::Uniform;
use crate::models::event::Event;
use crate::models::link::DocumentLink;
use crate::models::member::active_semester::{ActiveSemester, Enrollment};
use crate::models::member::Member;
use crate::models::minutes::Minutes;
use crate::models::money::{ClubTransaction, Fee};
use crate::models::permissions::{MemberRole, RolePermission};
use crate::models::semester::Semester;
use crate::models::song::{PublicSong, Song, SongLink};
use crate::models::static_data::StaticData;
use crate::models::variable::Variable;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn user<'c>(&self, ctx: &'c Context<'c>) -> Option<Member> {
        ctx.data_opt::<Member>().cloned()
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn member(&self, ctx: &Context<'_>, email: String) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        Member::with_email(&email, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn members(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] include_class: bool,
        #[graphql(default = true)] include_club: bool,
        #[graphql(default = false)] include_inactive: bool,
    ) -> Result<Vec<Member>> {
        let conn = DbConn::from_ctx(ctx);
        let semester = Semester::get_current(conn).await?;

        let mut selected_members = vec![];
        for member in Member::all(conn).await? {
            // TODO: optimize queries?
            let enrollment =
                ActiveSemester::for_member_during_semester(&member.email, &semester.name, conn)
                    .await?
                    .map(|s| s.enrollment);
            let include_member = match enrollment {
                Some(Enrollment::Class) => include_class,
                Some(Enrollment::Club) => include_club,
                None => include_inactive,
            };

            if include_member {
                selected_members.push(member);
            }
        }

        Ok(selected_members)
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn event(&self, ctx: &Context<'_>, id: i32) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        Event::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn events(&self, ctx: &Context<'_>) -> Result<Vec<Event>> {
        let conn = DbConn::from_ctx(ctx);
        let semester = Semester::get_current(conn).await?;
        Event::for_semester(&semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_ABSENCE_REQUESTS)")]
    pub async fn absence_requests(&self, ctx: &Context<'_>) -> Result<Vec<AbsenceRequest>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        AbsenceRequest::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_request(&self, ctx: &Context<'_>, id: i32) -> Result<GigRequest> {
        let conn = DbConn::from_ctx(ctx);
        GigRequest::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_requests(&self, ctx: &Context<'_>) -> Result<Vec<GigRequest>> {
        let conn = DbConn::from_ctx(ctx);
        GigRequest::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn meeting_minutes(&self, ctx: &Context<'_>, id: i32) -> Result<Minutes> {
        let conn = DbConn::from_ctx(ctx);
        Minutes::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn all_meeting_minutes(&self, ctx: &Context<'_>) -> Result<Vec<Minutes>> {
        let conn = DbConn::from_ctx(ctx);
        Minutes::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn current_semester(&self, ctx: &Context<'_>) -> Result<Semester> {
        let conn = DbConn::from_ctx(ctx);
        Semester::get_current(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let conn = DbConn::from_ctx(ctx);
        Semester::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semesters(&self, ctx: &Context<'_>) -> Result<Vec<Semester>> {
        let conn = DbConn::from_ctx(ctx);
        Semester::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniform(&self, ctx: &Context<'_>, id: i32) -> Result<Uniform> {
        let conn = DbConn::from_ctx(ctx);
        Uniform::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniforms(&self, ctx: &Context<'_>) -> Result<Vec<Uniform>> {
        let conn = DbConn::from_ctx(ctx);
        Uniform::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn links(&self, ctx: &Context<'_>) -> Result<Vec<DocumentLink>> {
        let conn = DbConn::from_ctx(ctx);
        DocumentLink::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song(&self, ctx: &Context<'_>, id: i32) -> Result<Song> {
        let conn = DbConn::from_ctx(ctx);
        Song::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn songs(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let conn = DbConn::from_ctx(ctx);
        Song::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song_link(&self, ctx: &Context<'_>, id: i32) -> Result<SongLink> {
        let conn = DbConn::from_ctx(ctx);
        SongLink::with_id(id, conn).await
    }

    pub async fn public_songs(&self, ctx: &Context<'_>) -> Result<Vec<PublicSong>> {
        let conn = DbConn::from_ctx(ctx);
        PublicSong::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn r#static(&self) -> StaticData {
        StaticData
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn fees(&self, ctx: &Context<'_>) -> Result<Vec<Fee>> {
        let conn = DbConn::from_ctx(ctx);
        Fee::all(conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn officers(&self, ctx: &Context<'_>) -> Result<Vec<MemberRole>> {
        let conn = DbConn::from_ctx(ctx);
        MemberRole::current_officers(conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn current_permissions(&self, ctx: &Context<'_>) -> Result<Vec<RolePermission>> {
        let conn = DbConn::from_ctx(ctx);
        RolePermission::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn variable(&self, ctx: &Context<'_>, key: String) -> Result<Variable> {
        // TODO: permissions?
        let conn = DbConn::from_ctx(ctx);
        Variable::with_key(&key, conn).await
    }
}
