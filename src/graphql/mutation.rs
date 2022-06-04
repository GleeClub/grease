use async_graphql::{Context, Object, Result};

use crate::db::DbConn;
use crate::graphql::guards::{LoggedIn, Permission};
use crate::graphql::SUCCESS_MESSAGE;
use crate::models::event::absence_request::{AbsenceRequest, AbsenceRequestState};
use crate::models::event::attendance::{Attendance, AttendanceUpdate};
use crate::models::event::carpool::{Carpool, UpdatedCarpool};
use crate::models::event::gig::{GigRequest, GigRequestStatus, NewGigRequest};
use crate::models::event::uniform::{NewUniform, Uniform};
use crate::models::event::{Event, NewEvent};
use crate::models::link::DocumentLink;
use crate::models::member::active_semester::ActiveSemester;
use crate::models::member::session::Session;
use crate::models::member::{Member, MemberUpdate, NewMember, RegisterForSemesterForm};
use crate::models::minutes::{Minutes, UpdatedMeetingMinutes};
use crate::models::money::{ClubTransaction, Fee, TransactionBatch};
use crate::models::permissions::{MemberRole, NewRolePermission, RolePermission};
use crate::models::semester::{NewSemester, Semester};
use crate::models::song::{NewSong, NewSongLink, Song, SongLink, SongLinkUpdate, SongUpdate};
use crate::models::variable::Variable;

// TODO: sendEmail(since: NaiveDateTime!): Boolean!

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Gets a login token on successful login
    pub async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        pass_hash: String,
    ) -> Result<String> {
        let conn = DbConn::from_ctx(ctx);
        if !Member::login_is_valid(&email, &pass_hash, conn).await? {
            return Err("Invalid email or password".into());
        }

        Session::get_or_generate_token(&email, conn).await
    }

    /// Logs the member out
    pub async fn logout(&self, ctx: &Context<'_>) -> Result<&'static str> {
        let user = ctx
            .data_opt::<Member>()
            .ok_or_else(|| "Not currently logged in")?;
        let conn = DbConn::from_ctx(ctx);
        Session::remove(&user.email, conn).await?;

        Ok(SUCCESS_MESSAGE)
    }

    pub async fn forgot_password(&self, ctx: &Context<'_>, email: String) -> Result<&'static str> {
        let conn = DbConn::from_ctx(ctx);
        Session::generate_for_forgotten_password(&email, conn).await?;

        Ok(SUCCESS_MESSAGE)
    }

    pub async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        pass_hash: String,
    ) -> Result<&'static str> {
        let conn = DbConn::from_ctx(ctx);
        Session::reset_password(&token, &pass_hash, conn).await?;

        Ok(SUCCESS_MESSAGE)
    }

    pub async fn register_member(
        &self,
        ctx: &Context<'_>,
        new_member: NewMember,
    ) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        let email = new_member.email.clone();
        Member::register(new_member, conn).await?;

        Member::with_email(&email, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn register_for_semester(
        &self,
        ctx: &Context<'_>,
        new_semester: RegisterForSemesterForm,
    ) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        Member::register_for_current_semester(user.email.clone(), new_semester, conn).await?;

        Member::with_email(&user.email, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn update_profile(
        &self,
        ctx: &Context<'_>,
        new_member: MemberUpdate,
    ) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        let new_email = new_member.email.clone();
        Member::update(&user.email, new_member, true, conn).await?;

        Member::with_email(&new_email, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_USER)")]
    pub async fn update_member(
        &self,
        ctx: &Context<'_>,
        email: String,
        new_member: MemberUpdate,
    ) -> Result<Member> {
        let conn = DbConn::from_ctx(ctx);
        let new_email = new_member.email.clone();
        Member::update(&email, new_member, false, conn).await?;

        Member::with_email(&new_email, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::SWITCH_USER)")]
    pub async fn login_as(&self, ctx: &Context<'_>, email: String) -> Result<String> {
        let conn = DbConn::from_ctx(ctx);

        Session::get_or_generate_token(&email, conn).await
    }

    /// Deletes a member and returns their email
    #[graphql(guard = "LoggedIn.and(Permission::DELETE_USER)")]
    pub async fn delete_member(&self, ctx: &Context<'_>, email: String) -> Result<String> {
        let conn = DbConn::from_ctx(ctx);
        Member::delete(&email, conn).await?;

        Ok(email)
    }

    #[graphql(guard = "LoggedIn.and(Permission::CREATE_EVENT.for_type(&new_event.event.r#type))")]
    pub async fn create_event(&self, ctx: &Context<'_>, new_event: NewEvent) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = Event::create(new_event, None, conn).await?;

        Event::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::MODIFY_EVENT.for_type(&new_event.event.r#type))")]
    pub async fn update_event(
        &self,
        ctx: &Context<'_>,
        id: i32,
        new_event: NewEvent,
    ) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        Event::update(id, new_event, conn).await?;

        Event::with_id(id, conn).await
    }

    // TODO: event type
    /// Deletes an event and returns its ID
    #[graphql(guard = "LoggedIn.and(Permission::DELETE_EVENT)")]
    pub async fn delete_event(&self, ctx: &Context<'_>, id: i32) -> Result<i32> {
        let conn = DbConn::from_ctx(ctx);
        Event::delete(id, conn).await?;

        Ok(id)
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn update_attendance(
        &self,
        ctx: &Context<'_>,
        event_id: i32,
        email: String,
        update: AttendanceUpdate,
    ) -> Result<Attendance> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        let event = Event::with_id(event_id, conn).await?;

        if !Permission::EDIT_ATTENDANCE
            .for_type(&event.r#type)
            .granted_to(&user.email, conn)
            .await?
        {
            let user_section =
                ActiveSemester::for_member_during_semester(&user.email, &event.semester, conn)
                    .await?
                    .map(|semester| semester.section);
            let member_section =
                ActiveSemester::for_member_during_semester(&email, &event.semester, conn)
                    .await?
                    .map(|semester| semester.section);

            if user_section.is_none()
                || user_section != member_section
                || !Permission::EDIT_ATTENDANCE_OWN_SECTION
                    .for_type(&event.r#type)
                    .granted_to(&user.email, conn)
                    .await?
            {
                // TODO: use the normal format?
                return Err("Not allowed to edit attendance".into());
            }
        }

        Attendance::update(event_id, &email, update, conn).await?;

        Attendance::for_member_at_event(&email, event_id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn rsvp_for_event(
        &self,
        ctx: &Context<'_>,
        id: i32,
        attending: bool,
    ) -> Result<Attendance> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        Attendance::rsvp_for_event(id, &user.email, attending, conn).await?;

        Attendance::for_member_at_event(&user.email, id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn confirm_for_event(&self, ctx: &Context<'_>, id: i32) -> Result<Attendance> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        Attendance::confirm_for_event(id, &user.email, conn).await?;

        Attendance::for_member_at_event(&user.email, id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_CARPOOLS)")]
    pub async fn update_carpools(
        &self,
        ctx: &Context<'_>,
        event_id: i32,
        carpools: Vec<UpdatedCarpool>,
    ) -> Result<Vec<Carpool>> {
        let conn = DbConn::from_ctx(ctx);
        Carpool::update(event_id, carpools, conn).await?;

        Carpool::for_event(event_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_ABSENCE_REQUESTS)")]
    pub async fn respond_to_absence_request(
        &self,
        ctx: &Context<'_>,
        event_id: i32,
        email: String,
        approved: bool,
    ) -> Result<AbsenceRequest> {
        let conn = DbConn::from_ctx(ctx);
        let state = if approved {
            AbsenceRequestState::Approved
        } else {
            AbsenceRequestState::Denied
        };

        AbsenceRequest::set_state(event_id, &email, state, conn).await?;

        AbsenceRequest::for_member_at_event(&email, event_id, conn).await
    }

    #[graphql(guard = "LoggedIn")]
    pub async fn submit_absence_request(
        &self,
        ctx: &Context<'_>,
        event_id: i32,
        reason: String,
    ) -> Result<AbsenceRequest> {
        let conn = DbConn::from_ctx(ctx);
        let user = ctx.data_unchecked::<Member>();
        AbsenceRequest::submit(event_id, &user.email, &reason, conn).await?;

        AbsenceRequest::for_member_at_event(&user.email, event_id, conn).await
    }

    pub async fn submit_gig_request(
        &self,
        ctx: &Context<'_>,
        request: NewGigRequest,
    ) -> Result<GigRequest> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = GigRequest::submit(request, conn).await?;

        GigRequest::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn dismiss_gig_request(&self, ctx: &Context<'_>, id: i32) -> Result<GigRequest> {
        let conn = DbConn::from_ctx(ctx);
        GigRequest::set_status(id, GigRequestStatus::Dismissed, conn).await?;

        GigRequest::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn reopen_gig_request(&self, ctx: &Context<'_>, id: i32) -> Result<GigRequest> {
        let conn = DbConn::from_ctx(ctx);
        GigRequest::set_status(id, GigRequestStatus::Pending, conn).await?;

        GigRequest::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::CREATE_EVENT.for_type(&new_event.event.r#type))")]
    pub async fn create_event_from_gig_request(
        &self,
        ctx: &Context<'_>,
        request_id: i32,
        new_event: NewEvent,
    ) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        let request = GigRequest::with_id(request_id, conn).await?;
        let new_id = Event::create(new_event, Some(request), conn).await?;

        Event::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_LINKS)")]
    pub async fn create_link(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: String,
    ) -> Result<DocumentLink> {
        let conn = DbConn::from_ctx(ctx);
        DocumentLink::create(&name, &url, conn).await?;

        DocumentLink::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_LINKS)")]
    pub async fn delete_link(&self, ctx: &Context<'_>, name: String) -> Result<DocumentLink> {
        let conn = DbConn::from_ctx(ctx);
        let link = DocumentLink::with_name(&name, conn).await?;
        DocumentLink::delete(&name, conn).await?;

        Ok(link)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn create_semester(
        &self,
        ctx: &Context<'_>,
        new_semester: NewSemester,
    ) -> Result<Semester> {
        let conn = DbConn::from_ctx(ctx);
        let name = new_semester.name.clone();
        Semester::create(new_semester, conn).await?;

        Semester::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn update_semester(
        &self,
        ctx: &Context<'_>,
        name: String,
        update: NewSemester,
    ) -> Result<Semester> {
        let conn = DbConn::from_ctx(ctx);
        let new_name = update.name.clone();
        Semester::update(&name, update, conn).await?;

        Semester::with_name(&new_name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn set_current_semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let conn = DbConn::from_ctx(ctx);
        Semester::set_current(&name, conn).await?;

        Semester::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn create_meeting_minutes(&self, ctx: &Context<'_>, name: String) -> Result<Minutes> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = Minutes::create(&name, conn).await?;

        Minutes::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn update_meeting_minutes(
        &self,
        ctx: &Context<'_>,
        id: i32,
        update: UpdatedMeetingMinutes,
    ) -> Result<Minutes> {
        let conn = DbConn::from_ctx(ctx);
        Minutes::update(id, update, conn).await?;

        Minutes::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn email_meeting_minutes(&self, ctx: &Context<'_>, id: i32) -> Result<Minutes> {
        let conn = DbConn::from_ctx(ctx);

        // TODO: implement emails

        Minutes::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn delete_meeting_minutes(&self, ctx: &Context<'_>, id: i32) -> Result<Minutes> {
        let conn = DbConn::from_ctx(ctx);
        let minutes = Minutes::with_id(id, conn).await?;
        Minutes::delete(id, conn).await?;

        Ok(minutes)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn create_uniform(
        &self,
        ctx: &Context<'_>,
        new_uniform: NewUniform,
    ) -> Result<Uniform> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = Uniform::create(new_uniform, conn).await?;

        Uniform::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn update_uniform(
        &self,
        ctx: &Context<'_>,
        id: i32,
        update: NewUniform,
    ) -> Result<Uniform> {
        let conn = DbConn::from_ctx(ctx);
        Uniform::update(id, update, conn).await?;

        Uniform::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn delete_uniform(&self, ctx: &Context<'_>, id: i32) -> Result<Uniform> {
        let conn = DbConn::from_ctx(ctx);
        let uniform = Uniform::with_id(id, conn).await?;
        Uniform::delete(id, conn).await?;

        Ok(uniform)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn create_song(&self, ctx: &Context<'_>, new_song: NewSong) -> Result<Song> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = Song::create(new_song, conn).await?;

        Song::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn update_song(
        &self,
        ctx: &Context<'_>,
        id: i32,
        update: SongUpdate,
    ) -> Result<Song> {
        let conn = DbConn::from_ctx(ctx);
        Song::update(id, update, conn).await?;

        Song::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn delete_song(&self, ctx: &Context<'_>, id: i32) -> Result<Song> {
        let conn = DbConn::from_ctx(ctx);
        let song = Song::with_id(id, conn).await?;
        Song::delete(id, conn).await?;

        Ok(song)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn create_song_link(
        &self,
        ctx: &Context<'_>,
        song_id: i32,
        new_link: NewSongLink,
    ) -> Result<SongLink> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = SongLink::create(song_id, new_link, conn).await?;

        SongLink::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn update_song_link(
        &self,
        ctx: &Context<'_>,
        id: i32,
        update: SongLinkUpdate,
    ) -> Result<SongLink> {
        let conn = DbConn::from_ctx(ctx);
        SongLink::update(id, update, conn).await?;

        SongLink::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn delete_song_link(&self, ctx: &Context<'_>, id: i32) -> Result<SongLink> {
        let conn = DbConn::from_ctx(ctx);
        let link = SongLink::with_id(id, conn).await?;
        SongLink::delete(id, conn).await?;

        Ok(link)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_PERMISSIONS)")]
    pub async fn add_permission_to_role(
        &self,
        ctx: &Context<'_>,
        role_permission: NewRolePermission,
    ) -> Result<bool> {
        let conn = DbConn::from_ctx(ctx);
        RolePermission::add(role_permission, conn).await?;

        Ok(true)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_PERMISSIONS)")]
    pub async fn remove_permission_from_role(
        &self,
        ctx: &Context<'_>,
        role_permission: NewRolePermission,
    ) -> Result<bool> {
        let conn = DbConn::from_ctx(ctx);
        RolePermission::remove(role_permission, conn).await?;

        Ok(true)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn add_officership(
        &self,
        ctx: &Context<'_>,
        role: String,
        email: String,
    ) -> Result<bool> {
        let conn = DbConn::from_ctx(ctx);
        MemberRole::add(&email, &role, conn).await?;

        Ok(true)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn remove_officership(
        &self,
        ctx: &Context<'_>,
        role: String,
        email: String,
    ) -> Result<bool> {
        let conn = DbConn::from_ctx(ctx);
        MemberRole::remove(&email, &role, conn).await?;

        Ok(true)
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn update_fee_amount(
        &self,
        ctx: &Context<'_>,
        name: String,
        amount: i32,
    ) -> Result<Fee> {
        let conn = DbConn::from_ctx(ctx);
        Fee::set_amount(&name, amount, conn).await?;

        Fee::with_name(&name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn charge_dues(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        Fee::charge_dues_for_semester(conn).await?;

        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn charge_late_dues(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        Fee::charge_late_dues_for_semester(conn).await?;

        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn add_batch_of_transactions(
        &self,
        ctx: &Context<'_>,
        batch: TransactionBatch,
    ) -> Result<Vec<ClubTransaction>> {
        let conn = DbConn::from_ctx(ctx);
        let current_semester = Semester::get_current(conn).await?;
        ClubTransaction::add_batch(batch, conn).await?;

        ClubTransaction::for_semester(&current_semester.name, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn resolve_transaction(
        &self,
        ctx: &Context<'_>,
        id: i32,
        resolved: bool,
    ) -> Result<ClubTransaction> {
        let conn = DbConn::from_ctx(ctx);
        ClubTransaction::resolve(id, resolved, conn).await?;

        ClubTransaction::with_id(id, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn set_variable(
        &self,
        ctx: &Context<'_>,
        key: String,
        value: String,
    ) -> Result<Variable> {
        let conn = DbConn::from_ctx(ctx);
        Variable::set(&key, &value, conn).await?;

        Variable::with_key(&key, conn).await
    }

    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn unset_variable(&self, ctx: &Context<'_>, key: String) -> Result<String> {
        let conn = DbConn::from_ctx(ctx);
        let variable = Variable::with_key(&key, conn).await?;
        Variable::unset(&key, conn).await?;

        Ok(variable.value)
    }
}
