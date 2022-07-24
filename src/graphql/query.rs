use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::graphql::guards::{LoggedIn, Permission};
use crate::models::event::absence_request::AbsenceRequest;
use crate::models::event::gig::GigRequest;
use crate::models::event::public::PublicEvent;
use crate::models::event::uniform::Uniform;
use crate::models::event::Event;
use crate::models::link::DocumentLink;
use crate::models::member::{IncludeContext, Member};
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
        let pool: &PgPool = ctx.data_unchecked();
        Member::with_email(&email, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn members(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] include_class: bool,
        #[graphql(default = true)] include_club: bool,
        #[graphql(default = false)] include_inactive: bool,
    ) -> Result<Vec<Member>> {
        let pool: &PgPool = ctx.data_unchecked();
        let semester = Semester::get_current(pool).await?;
        let included = IncludeContext {
            class: include_class,
            club: include_club,
            inactive: include_inactive,
        };

        Member::all_included(included, &semester.name, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn event(&self, ctx: &Context<'_>, id: i64) -> Result<Event> {
        let pool: &PgPool = ctx.data_unchecked();
        Event::with_id(id, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn events(&self, ctx: &Context<'_>) -> Result<Vec<Event>> {
        let pool: &PgPool = ctx.data_unchecked();
        let semester = Semester::get_current(pool).await?;
        Event::for_semester(&semester.name, pool).await
    }

    pub async fn public_events(&self, ctx: &Context<'_>) -> Result<Vec<PublicEvent>> {
        let pool: &PgPool = ctx.data_unchecked();
        PublicEvent::all_for_current_semester(pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_ABSENCE_REQUESTS)")]
    pub async fn absence_requests(&self, ctx: &Context<'_>) -> Result<Vec<AbsenceRequest>> {
        let pool: &PgPool = ctx.data_unchecked();
        let current_semester = Semester::get_current(pool).await?;
        AbsenceRequest::for_semester(&current_semester.name, pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_request(&self, ctx: &Context<'_>, id: i64) -> Result<GigRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        GigRequest::with_id(id, pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn gig_requests(&self, ctx: &Context<'_>) -> Result<Vec<GigRequest>> {
        let pool: &PgPool = ctx.data_unchecked();
        GigRequest::all(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn meeting_minutes(&self, ctx: &Context<'_>, id: i64) -> Result<Minutes> {
        let pool: &PgPool = ctx.data_unchecked();
        Minutes::with_id(id, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn all_meeting_minutes(&self, ctx: &Context<'_>) -> Result<Vec<Minutes>> {
        let pool: &PgPool = ctx.data_unchecked();
        Minutes::all(pool).await
    }

    pub async fn current_semester(&self, ctx: &Context<'_>) -> Result<Semester> {
        let pool: &PgPool = ctx.data_unchecked();
        Semester::get_current(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let pool: &PgPool = ctx.data_unchecked();
        Semester::with_name(&name, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semesters(&self, ctx: &Context<'_>) -> Result<Vec<Semester>> {
        let pool: &PgPool = ctx.data_unchecked();
        Semester::all(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniform(&self, ctx: &Context<'_>, id: i64) -> Result<Uniform> {
        let pool: &PgPool = ctx.data_unchecked();
        Uniform::with_id(id, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniforms(&self, ctx: &Context<'_>) -> Result<Vec<Uniform>> {
        let pool: &PgPool = ctx.data_unchecked();
        Uniform::all(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn links(&self, ctx: &Context<'_>) -> Result<Vec<DocumentLink>> {
        let pool: &PgPool = ctx.data_unchecked();
        DocumentLink::all(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song(&self, ctx: &Context<'_>, id: i64) -> Result<Song> {
        let pool: &PgPool = ctx.data_unchecked();
        Song::with_id(id, pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn songs(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let pool: &PgPool = ctx.data_unchecked();
        Song::all(pool).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song_link(&self, ctx: &Context<'_>, id: i64) -> Result<SongLink> {
        let pool: &PgPool = ctx.data_unchecked();
        SongLink::with_id(id, pool).await
    }

    pub async fn public_songs(&self, ctx: &Context<'_>) -> Result<Vec<PublicSong>> {
        let pool: &PgPool = ctx.data_unchecked();
        PublicSong::all(pool).await
    }

    pub async fn r#static(&self) -> StaticData {
        StaticData
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let pool: &PgPool = ctx.data_unchecked();
        let current_semester = Semester::get_current(pool).await?;
        ClubTransaction::for_semester(&current_semester.name, pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::VIEW_TRANSACTIONS)")]
    pub async fn fees(&self, ctx: &Context<'_>) -> Result<Vec<Fee>> {
        let pool: &PgPool = ctx.data_unchecked();
        Fee::all(pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn officers(&self, ctx: &Context<'_>) -> Result<Vec<MemberRole>> {
        let pool: &PgPool = ctx.data_unchecked();
        MemberRole::current_officers(pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn current_permissions(&self, ctx: &Context<'_>) -> Result<Vec<RolePermission>> {
        let pool: &PgPool = ctx.data_unchecked();
        RolePermission::all(pool).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn variable(&self, ctx: &Context<'_>, key: String) -> Result<Variable> {
        let pool: &PgPool = ctx.data_unchecked();
        Variable::with_key(&key, pool).await
    }
}
