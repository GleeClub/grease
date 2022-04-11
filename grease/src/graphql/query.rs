use async_graphql::*;
use sqlx::MySqlPool;

use crate::graphql::LoggedIn;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn user(&self, ctx: Context<'_>) -> Option<Member> {
        ctx.data_opt::<Member>()
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn member(&self, ctx: &Context<'_>, email: String) -> Result<Member> {
        let conn = ctx.data_unchecked::<DbConn>();
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
        let conn = ctx.data_unchecked::<DbConn>();
        let semester = Semester::current(conn).await?;

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
    pub async fn event(&self, ctx: &Context<'_>, id: i64) -> Result<Event> {
        let conn = ctx.data_unchecked::<DbConn>();
        Event::load(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn events(&self, ctx: &Context<'_>) -> Result<Vec<Event>> {
        let conn = ctx.data_unchecked::<DbConn>();
        let semester = Semester::load_current(conn).await?;
        Event::load_all_for_semester(&semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::PROCESS_ABSENCE_REQUESTS")]
    pub async fn absence_requests(&self, ctx: &Context<'_>) -> Result<Vec<AbsenceRequest>> {
        let conn = ctx.data_unchecked::<DbConn>();
        AbsenceRequest::for_current_semester(conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::PROCESS_GIG_REQUESTS")]
    pub async fn gig_request(&self, ctx: &Context<'_>, id: i64) -> Result<GigRequest> {
        let conn = ctx.data_unchecked::<DbConn>();
        GigRequest::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::PROCESS_GIG_REQUESTS")]
    pub async fn gig_requests(&self, ctx: &Context<'_>) -> Result<Vec<GigRequest>> {
        let conn = ctx.data_unchecked::<DbConn>();
        GigRequest::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn meeting_minutes(&self, ctx: &Context<'_>, id: i64) -> Result<Minutes> {
        let conn = ctx.data_unchecked::<DbConn>();
        Minutes::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn all_meeting_minutes(&self, ctx: &Context<'_>) -> Result<Vec<Minutes>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Minutes::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn current_semester(&self, ctx: &Context<'_>) -> Result<Semester> {
        let conn = ctx.data_unchecked::<DbConn>();
        Semester::current(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let conn = ctx.data_unchecked::<DbConn>();
        Semester::with_name(name, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn semesters(&self, ctx: &Context<'_>) -> Result<Vec<Semester>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Semester::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniform(&self, ctx: &Context<'_>, id: i64) -> Result<Uniform> {
        let conn = ctx.data_unchecked::<DbConn>();
        Uniform::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn uniforms(&self, ctx: &Context<'_>) -> Result<Vec<Uniform>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Uniform::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn documents(&self, ctx: &Context<'_>) -> Result<Vec<Document>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Document::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song(&self, ctx: &Context<'_>, id: i64) -> Result<Song> {
        let conn = ctx.data_unchecked::<DbConn>();
        Song::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn songs(&self, ctx: &Context<'_>) -> Result<Vec<Song>> {
        let conn = ctx.data_unchecked::<DbConn>();
        Song::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn song_link(&self, ctx: &Context<'_>, id: i64) -> Result<SongLink> {
        let conn = ctx.data_unchecked::<DbConn>();
        SongLink::with_id(id, conn).await
    }

    pub async fn public_songs(&self, ctx: &Context<'_>) -> Result<Vec<PublicSong>> {
        let conn = ctx.data_unchecked::<DbConn>();
        PublicSong::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn r#static(&self, ctx: &Context<'_>) -> StaticData {
        StaticData
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::VIEW_TRANSACTIONS")]
    pub async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let conn = ctx.data_unchecked::<DbConn>();
        let current_semester = Semester::current(conn).await?;
        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::VIEW_TRANSACTIONS")]
    pub async fn fees(&self, ctx: &Context<'_>) -> Result<SongLink> {
        let conn = ctx.data_unchecked::<DbConn>();
        Fee::all(conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::EDIT_OFFICERS")]
    pub async fn officers(&self, ctx: &Context<'_>) -> Result<Vec<MemberRole>> {
        let conn = ctx.data_unchecked::<DbConn>();
        MemberRole::current_officers(conn).await
    }

    #[graphql(guard = "LoggedIn", guard = "Permission::EDIT_OFFICERS")]
    pub async fn current_permissions(&self, ctx: &Context<'_>) -> Result<Vec<RolePermission>> {
        let conn = ctx.data_unchecked::<DbConn>();
        RolePermission::all(conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn variable(&self, ctx: &Context<'_>, key: String) -> Result<Variable> {
        let conn = ctx.data_unchecked::<DbConn>();
        Variables::with_key(key, conn).await
    }
}
