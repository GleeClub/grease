use async_graphql::*;
use crate::graphql::LoggedIn;
use crate::graphql::permission::Permission;
use crate::db::get_conn;
use crate::models::minutes::Minutes;
use crate::models::document::Document;
use crate::models::song::{Song, PublicSong, SongLink};
use crate::models::static_data::StaticData;
use crate::models::member::Member;
use crate::models::semester::Semester;
use crate::models::event::Event;
use crate::models::member::active_semester::Enrollment;
use crate::models::event::absence_request::AbsenceRequest;
use crate::models::event::gig::GigRequest;
use crate::models::event::uniform::Uniform;
use crate::models::money::{Fee, ClubTransaction};
use crate::models::permissions::{MemberRole, RolePermission};
use crate::models::variable::Variable;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn user(&self, ctx: &Context<'_>) -> Option<Member> {
        ctx.data_opt::<Member>()
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn member(&self, ctx: &Context<'_>, email: String) -> Result<Member> {
        Member::with_email(&email, get_conn(ctx)).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn members(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] include_class: bool,
        #[graphql(default = true)] include_club: bool,
        #[graphql(default = false)] include_inactive: bool,
    ) -> Result<Vec<Member>> {
        let mut conn = get_conn(ctx);
        let semester = Semester::get_current(conn).await?;

        let mut selected_members = vec![];
        for member in Member::load_all(conn).await? {
            // TODO: optimize queries?
            let enrollment = member
                .load_semester(semester.name)
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
        let mut conn = get_conn(ctx);
        Event::load(id, &mut conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn events(&self, ctx: &Context<'_>) -> Result<Vec<Event>> {
        let mut conn = get_conn(ctx);
        let semester = Semester::load_current(&mut conn).await?;
        Event::load_all_for_semester(&semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_ABSENCE_REQUESTS)")]
    pub async fn absence_requests(&self, ctx: &Context<'_>) -> Result<Vec<AbsenceRequest>> {
        let mut conn = get_conn(ctx);
        AbsenceRequest::for_current_semester(conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_request(&self, ctx: &Context<'_>, id: i32) -> Result<GigRequest> {
        let mut conn = get_conn(ctx);
        GigRequest::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_requests(&self, ctx: &Context<'_>) -> Result<Vec<GigRequest>> {
        let mut conn = get_conn(ctx);
        GigRequest::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn meeting_minutes(&self, ctx: &Context<'_>, id: i32) -> Result<Minutes> {
        let mut conn = get_conn(ctx);
        Minutes::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn all_meeting_minutes(&self, ctx: &Context<'_>) -> Result<Vec<Minutes>> {
        let mut conn = get_conn(ctx);
        Minutes::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn current_semester(&self, ctx: &Context<'_>) -> Result<Semester> {
        let mut conn = get_conn(ctx);
        Semester::get_current(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let mut conn = get_conn(ctx);
        Semester::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semesters(&self, ctx: &Context<'_>) -> Result<Vec<Semester>> {
        let mut conn = get_conn(ctx);
        Semester::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniform(&self, ctx: &Context<'_>, id: i32) -> Result<Uniform> {
        let mut conn = get_conn(ctx);
        Uniform::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniforms(&self, ctx: &Context<'_>) -> Result<Vec<Uniform>> {
        let mut conn = get_conn(ctx);
        Uniform::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn documents(&self, ctx: &Context<'_>) -> Result<Vec<Document>> {
        let mut conn = get_conn(ctx);
        Document::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song(&self, ctx: &Context<'_>, id: i32) -> Result<Song> {
        let mut conn = get_conn(ctx);
        Song::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn songs(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let mut conn = get_conn(ctx);
        Song::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song_link(&self, ctx: &Context<'_>, id: i32) -> Result<SongLink> {
        let mut conn = get_conn(ctx);
        SongLink::with_id(id, conn).await
    }

    pub async fn public_songs(&self, ctx: &Context<'_>) -> Result<Vec<PublicSong>> {
        let mut conn = get_conn(ctx);
        PublicSong::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn r#static(&self, ctx: &Context<'_>) -> StaticData {
        StaticData
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let mut conn = get_conn(ctx);
        let current_semester = Semester::get_current(conn).await?;
        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn fees(&self, ctx: &Context<'_>) -> Result<SongLink> {
        let mut conn = get_conn(ctx);
        Fee::all(conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn officers(&self, ctx: &Context<'_>) -> Result<Vec<MemberRole>> {
        let mut conn = get_conn(ctx);
        MemberRole::current_officers(conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn current_permissions(&self, ctx: &Context<'_>) -> Result<Vec<RolePermission>> {
        let mut conn = get_conn(ctx);
        RolePermission::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn variable(&self, ctx: &Context<'_>, key: String) -> Result<Variable> {
        let mut conn = get_conn(ctx);
        Variables::with_key(key, conn).await
    }
}
