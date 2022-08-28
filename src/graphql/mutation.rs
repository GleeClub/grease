use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::email::event::NewEventEmail;
use crate::email::send_email;
use crate::graphql::guards::{LoggedIn, Permission};
use crate::graphql::SUCCESS_MESSAGE;
use crate::models::event::absence_request::{AbsenceRequest, AbsenceRequestStatus};
use crate::models::event::attendance::{Attendance, AttendanceUpdate};
use crate::models::event::carpool::{Carpool, UpdatedCarpool};
use crate::models::event::gig::{GigRequest, GigRequestStatus, NewGigRequest};
use crate::models::event::uniform::{NewUniform, Uniform};
use crate::models::event::{Event, NewEvent};
use crate::models::link::DocumentLink;
use crate::models::member::active_semester::ActiveSemester;
use crate::models::member::session::{PasswordReset, Session};
use crate::models::member::{Member, MemberUpdate, NewMember, RegisterForSemesterForm};
use crate::models::minutes::{Minutes, UpdatedMeetingMinutes};
use crate::models::money::{ClubTransaction, Fee, TransactionBatch};
use crate::models::permissions::{MemberRole, NewRolePermission, RolePermission};
use crate::models::semester::{NewSemester, Semester};
use crate::models::song::{NewSong, NewSongLink, Song, SongLink, SongLinkUpdate, SongUpdate};
use crate::models::variable::Variable;

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
        let pool: &PgPool = ctx.data_unchecked();
        if !Member::login_is_valid(&email, &pass_hash, pool).await? {
            return Err("Invalid email or password".into());
        }

        Session::get_or_generate_token(&email, pool).await
    }

    /// Logs the member out
    pub async fn logout(&self, ctx: &Context<'_>) -> Result<&'static str> {
        let user = ctx.data_opt::<Member>().ok_or("Not currently logged in")?;
        let pool: &PgPool = ctx.data_unchecked();
        Session::remove(&user.email, pool).await?;

        Ok(SUCCESS_MESSAGE)
    }

    /// Requests a password reset email for the given member
    pub async fn forgot_password(&self, ctx: &Context<'_>, email: String) -> Result<&'static str> {
        let pool: &PgPool = ctx.data_unchecked();
        PasswordReset::generate(&email, pool).await?;

        Ok(SUCCESS_MESSAGE)
    }

    /// Resets the member's password
    pub async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        pass_hash: String,
    ) -> Result<&'static str> {
        let pool: &PgPool = ctx.data_unchecked();
        PasswordReset::reset_from_token(&token, &pass_hash, pool).await?;

        Ok(SUCCESS_MESSAGE)
    }

    /// Registers a new member
    pub async fn register_member(
        &self,
        ctx: &Context<'_>,
        new_member: NewMember,
    ) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        let email = new_member.email.clone();
        Member::register(new_member, pool).await?;

        Member::with_email(&email, pool).await
    }

    /// Registers an existing member for the current semester
    #[graphql(guard = "LoggedIn")]
    pub async fn register_for_semester(
        &self,
        ctx: &Context<'_>,
        new_semester: RegisterForSemesterForm,
    ) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        Member::register_for_current_semester(user.email.clone(), new_semester, pool).await?;

        Member::with_email(&user.email, pool).await
    }

    /// Updates the current user's profile
    #[graphql(guard = "LoggedIn")]
    pub async fn update_profile(
        &self,
        ctx: &Context<'_>,
        new_member: MemberUpdate,
    ) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        let new_email = new_member.email.clone();
        Member::update(&user.email, new_member, true, pool).await?;

        Member::with_email(&new_email, pool).await
    }

    /// Updates the given member's profile
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_USER)")]
    pub async fn update_member(
        &self,
        ctx: &Context<'_>,
        email: String,
        new_member: MemberUpdate,
    ) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_email = new_member.email.clone();
        Member::update(&email, new_member, false, pool).await?;

        Member::with_email(&new_email, pool).await
    }

    /// Logs in as the given member
    #[graphql(guard = "LoggedIn.and(Permission::SWITCH_USER)")]
    pub async fn login_as(&self, ctx: &Context<'_>, email: String) -> Result<String> {
        let pool: &PgPool = ctx.data_unchecked();

        Session::get_or_generate_token(&email, pool).await
    }

    /// Deletes a member and returns their email
    #[graphql(guard = "LoggedIn.and(Permission::DELETE_USER)")]
    pub async fn delete_member(&self, ctx: &Context<'_>, email: String) -> Result<String> {
        let pool: &PgPool = ctx.data_unchecked();
        Member::delete(&email, pool).await?;

        Ok(email)
    }

    /// Creates a new event
    #[graphql(guard = "LoggedIn.and(Permission::CREATE_EVENT.for_type(&new_event.event.r#type))")]
    pub async fn create_event(
        &self,
        ctx: &Context<'_>,
        new_event: NewEvent,
        gig_request_id: Option<i64>,
    ) -> Result<Event> {
        let pool: &PgPool = ctx.data_unchecked();
        let gig_request = if let Some(request_id) = gig_request_id {
            Some(GigRequest::with_id(request_id, pool).await?)
        } else {
            None
        };
        let new_id = Event::create(new_event, gig_request, pool).await?;
        let event = Event::with_id(new_id, pool).await?;

        let email = NewEventEmail::for_event(&event, pool).await?;
        send_email(email).await?;

        Ok(event)
    }

    /// Updates the given event
    #[graphql(guard = "LoggedIn.and(Permission::MODIFY_EVENT.for_type(&new_event.event.r#type))")]
    pub async fn update_event(
        &self,
        ctx: &Context<'_>,
        id: i64,
        new_event: NewEvent,
    ) -> Result<Event> {
        let pool: &PgPool = ctx.data_unchecked();
        Event::update(id, new_event, pool).await?;

        Event::with_id(id, pool).await
    }

    /// Deletes an event and returns its ID
    #[graphql(guard = "LoggedIn")]
    pub async fn delete_event(&self, ctx: &Context<'_>, id: i64) -> Result<i64> {
        let user: &Member = ctx.data_unchecked();
        let pool: &PgPool = ctx.data_unchecked();
        let event = Event::with_id(id, pool).await?;

        Permission::DELETE_EVENT
            .for_type(event.r#type)
            .ensure_granted_to(&user.email, pool)
            .await?;

        Event::delete(id, pool).await?;

        Ok(id)
    }

    /// Updates the attendance for the given member at the given event
    #[graphql(guard = "LoggedIn")]
    pub async fn update_attendance(
        &self,
        ctx: &Context<'_>,
        event_id: i64,
        email: String,
        update: AttendanceUpdate,
    ) -> Result<Attendance> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        let event = Event::with_id(event_id, pool).await?;

        if !Permission::EDIT_ATTENDANCE
            .for_type(&event.r#type)
            .granted_to(&user.email, pool)
            .await?
        {
            let user_section =
                ActiveSemester::for_member_during_semester(&user.email, &event.semester, pool)
                    .await?
                    .map(|semester| semester.section);
            let member_section =
                ActiveSemester::for_member_during_semester(&email, &event.semester, pool)
                    .await?
                    .map(|semester| semester.section);

            if user_section.is_none()
                || user_section != member_section
                || !Permission::EDIT_ATTENDANCE_OWN_SECTION
                    .for_type(&event.r#type)
                    .granted_to(&user.email, pool)
                    .await?
            {
                return Err("Not allowed to edit attendance".into());
            }
        }

        Attendance::update(event_id, &email, update, pool).await?;

        Attendance::for_member_at_event(&email, event_id, pool).await
    }

    /// Excuses all unconfirmed members at the given event
    #[graphql(guard = "LoggedIn")]
    pub async fn excuse_unconfirmed_for_event(
        &self,
        ctx: &Context<'_>,
        event_id: i64,
    ) -> Result<&'static str> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        let event = Event::with_id(event_id, pool).await?;
        Permission::EDIT_ATTENDANCE
            .for_type(event.r#type)
            .ensure_granted_to(&user.email, pool)
            .await?;

        Attendance::excuse_unconfirmed(event_id, pool).await?;

        Ok(SUCCESS_MESSAGE)
    }

    /// RSVP's for the given event
    #[graphql(guard = "LoggedIn")]
    pub async fn rsvp_for_event(
        &self,
        ctx: &Context<'_>,
        id: i64,
        attending: bool,
    ) -> Result<Attendance> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        Attendance::rsvp_for_event(id, &user.email, attending, pool).await?;

        Attendance::for_member_at_event(&user.email, id, pool).await
    }

    /// Confirms attendance for the given event
    #[graphql(guard = "LoggedIn")]
    pub async fn confirm_for_event(&self, ctx: &Context<'_>, id: i64) -> Result<Attendance> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        Attendance::confirm_for_event(id, &user.email, pool).await?;

        Attendance::for_member_at_event(&user.email, id, pool).await
    }

    /// Updates the carpools for the given event
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_CARPOOLS)")]
    pub async fn update_carpools(
        &self,
        ctx: &Context<'_>,
        event_id: i64,
        carpools: Vec<UpdatedCarpool>,
    ) -> Result<Vec<Carpool>> {
        let pool: &PgPool = ctx.data_unchecked();
        Carpool::update(event_id, carpools, pool).await?;

        Carpool::for_event(event_id, pool).await
    }

    /// Responds to an absence request from the given member for the given event
    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_ABSENCE_REQUESTS)")]
    pub async fn respond_to_absence_request(
        &self,
        ctx: &Context<'_>,
        event_id: i64,
        email: String,
        approved: bool,
    ) -> Result<AbsenceRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        let state = if approved {
            AbsenceRequestStatus::Approved
        } else {
            AbsenceRequestStatus::Denied
        };

        AbsenceRequest::set_state(event_id, &email, state, pool).await?;

        AbsenceRequest::for_member_at_event(&email, event_id, pool).await
    }

    /// Submits a new absence request for the current user at the given event
    #[graphql(guard = "LoggedIn")]
    pub async fn submit_absence_request(
        &self,
        ctx: &Context<'_>,
        event_id: i64,
        reason: String,
    ) -> Result<AbsenceRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        let user = ctx.data_unchecked::<Member>();
        AbsenceRequest::submit(event_id, &user.email, &reason, pool).await?;

        AbsenceRequest::for_member_at_event(&user.email, event_id, pool).await
    }

    /// Submits a new gig request
    pub async fn submit_gig_request(
        &self,
        ctx: &Context<'_>,
        request: NewGigRequest,
    ) -> Result<GigRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_id = GigRequest::submit(request, pool).await?;

        GigRequest::with_id(new_id, pool).await
    }

    /// Dismisses a gig request
    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn dismiss_gig_request(&self, ctx: &Context<'_>, id: i64) -> Result<GigRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        GigRequest::set_status(id, GigRequestStatus::Dismissed, pool).await?;

        GigRequest::with_id(id, pool).await
    }

    /// Reopens a dismissed gig request
    #[graphql(guard = "LoggedIn.and(Permission::PROCESS_GIG_REQUESTS)")]
    pub async fn reopen_gig_request(&self, ctx: &Context<'_>, id: i64) -> Result<GigRequest> {
        let pool: &PgPool = ctx.data_unchecked();
        GigRequest::set_status(id, GigRequestStatus::Pending, pool).await?;

        GigRequest::with_id(id, pool).await
    }

    /// Creates a new document link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_LINKS)")]
    pub async fn create_link(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: String,
    ) -> Result<DocumentLink> {
        let pool: &PgPool = ctx.data_unchecked();
        DocumentLink::create(&name, &url, pool).await?;

        DocumentLink::with_name(&name, pool).await
    }

    /// Updates the given document link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_LINKS)")]
    pub async fn update_link(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: String,
    ) -> Result<DocumentLink> {
        let pool: &PgPool = ctx.data_unchecked();
        DocumentLink::set_url(&name, &url, pool).await?;

        DocumentLink::with_name(&name, pool).await
    }

    /// Deletes the given document link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_LINKS)")]
    pub async fn delete_link(&self, ctx: &Context<'_>, name: String) -> Result<DocumentLink> {
        let pool: &PgPool = ctx.data_unchecked();
        let link = DocumentLink::with_name(&name, pool).await?;
        DocumentLink::delete(&name, pool).await?;

        Ok(link)
    }

    /// Creates a new semester
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn create_semester(
        &self,
        ctx: &Context<'_>,
        new_semester: NewSemester,
    ) -> Result<Semester> {
        let pool: &PgPool = ctx.data_unchecked();
        let name = new_semester.name.clone();
        Semester::create(new_semester, pool).await?;

        Semester::with_name(&name, pool).await
    }

    /// Updates the given semester
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn update_semester(
        &self,
        ctx: &Context<'_>,
        name: String,
        update: NewSemester,
    ) -> Result<Semester> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_name = update.name.clone();
        Semester::update(&name, update, pool).await?;

        Semester::with_name(&new_name, pool).await
    }

    /// Set the given semester as the current semester
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_SEMESTER)")]
    pub async fn set_current_semester(&self, ctx: &Context<'_>, name: String) -> Result<Semester> {
        let pool: &PgPool = ctx.data_unchecked();
        Semester::set_current(&name, pool).await?;

        Semester::with_name(&name, pool).await
    }

    /// Create some new meeting minutes
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn create_meeting_minutes(&self, ctx: &Context<'_>, name: String) -> Result<Minutes> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_id = Minutes::create(&name, pool).await?;

        Minutes::with_id(new_id, pool).await
    }

    /// Updates the given meeting minutes
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn update_meeting_minutes(
        &self,
        ctx: &Context<'_>,
        id: i64,
        update: UpdatedMeetingMinutes,
    ) -> Result<Minutes> {
        let pool: &PgPool = ctx.data_unchecked();
        Minutes::update(id, update, pool).await?;

        Minutes::with_id(id, pool).await
    }

    /// Deletes the given meeting minutes
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_MINUTES)")]
    pub async fn delete_meeting_minutes(&self, ctx: &Context<'_>, id: i64) -> Result<Minutes> {
        let pool: &PgPool = ctx.data_unchecked();
        let minutes = Minutes::with_id(id, pool).await?;
        Minutes::delete(id, pool).await?;

        Ok(minutes)
    }

    /// Creates a new uniform
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn create_uniform(
        &self,
        ctx: &Context<'_>,
        new_uniform: NewUniform,
    ) -> Result<Uniform> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_id = Uniform::create(new_uniform, pool).await?;

        Uniform::with_id(new_id, pool).await
    }

    /// Updates the given uniform
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn update_uniform(
        &self,
        ctx: &Context<'_>,
        id: i64,
        update: NewUniform,
    ) -> Result<Uniform> {
        let pool: &PgPool = ctx.data_unchecked();
        Uniform::update(id, update, pool).await?;

        Uniform::with_id(id, pool).await
    }

    /// Deletes the given uniform
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_UNIFORMS)")]
    pub async fn delete_uniform(&self, ctx: &Context<'_>, id: i64) -> Result<Uniform> {
        let pool: &PgPool = ctx.data_unchecked();
        let uniform = Uniform::with_id(id, pool).await?;
        Uniform::delete(id, pool).await?;

        Ok(uniform)
    }

    /// Creates a new song
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn create_song(&self, ctx: &Context<'_>, new_song: NewSong) -> Result<Song> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_id = Song::create(new_song, pool).await?;

        Song::with_id(new_id, pool).await
    }

    /// Updates the given song
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn update_song(
        &self,
        ctx: &Context<'_>,
        id: i64,
        update: SongUpdate,
    ) -> Result<Song> {
        let pool: &PgPool = ctx.data_unchecked();
        Song::update(id, update, pool).await?;

        Song::with_id(id, pool).await
    }

    /// Deletes the given song
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn delete_song(&self, ctx: &Context<'_>, id: i64) -> Result<Song> {
        let pool: &PgPool = ctx.data_unchecked();
        let song = Song::with_id(id, pool).await?;
        Song::delete(id, pool).await?;

        Ok(song)
    }

    /// Creates a new song link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn create_song_link(
        &self,
        ctx: &Context<'_>,
        song_id: i64,
        new_link: NewSongLink,
    ) -> Result<SongLink> {
        let pool: &PgPool = ctx.data_unchecked();
        let new_id = SongLink::create(song_id, new_link, pool).await?;

        SongLink::with_id(new_id, pool).await
    }

    /// Updates the given song link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn update_song_link(
        &self,
        ctx: &Context<'_>,
        id: i64,
        update: SongLinkUpdate,
    ) -> Result<SongLink> {
        let pool: &PgPool = ctx.data_unchecked();
        SongLink::update(id, update, pool).await?;

        SongLink::with_id(id, pool).await
    }

    /// Deletes the given song link
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_REPERTOIRE)")]
    pub async fn delete_song_link(&self, ctx: &Context<'_>, id: i64) -> Result<SongLink> {
        let pool: &PgPool = ctx.data_unchecked();
        let link = SongLink::with_id(id, pool).await?;
        SongLink::delete(id, pool).await?;

        Ok(link)
    }

    /// Adds a permission to the given role
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_PERMISSIONS)")]
    pub async fn add_permission_to_role(
        &self,
        ctx: &Context<'_>,
        role_permission: NewRolePermission,
    ) -> Result<bool> {
        let pool: &PgPool = ctx.data_unchecked();
        RolePermission::add(role_permission, pool).await?;

        Ok(true)
    }

    /// Removes a permission from the given role
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_PERMISSIONS)")]
    pub async fn remove_permission_from_role(
        &self,
        ctx: &Context<'_>,
        role_permission: NewRolePermission,
    ) -> Result<bool> {
        let pool: &PgPool = ctx.data_unchecked();
        RolePermission::remove(role_permission, pool).await?;

        Ok(true)
    }

    /// Assign the given member the given officer position
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn add_officership(
        &self,
        ctx: &Context<'_>,
        role: String,
        email: String,
    ) -> Result<bool> {
        let pool: &PgPool = ctx.data_unchecked();
        MemberRole::add(&email, &role, pool).await?;

        Ok(true)
    }

    /// Remove an officer position from the given mmember
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn remove_officership(
        &self,
        ctx: &Context<'_>,
        role: String,
        email: String,
    ) -> Result<bool> {
        let pool: &PgPool = ctx.data_unchecked();
        MemberRole::remove(&email, &role, pool).await?;

        Ok(true)
    }

    /// Update the cost for the given fee
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn update_fee_amount(
        &self,
        ctx: &Context<'_>,
        name: String,
        amount: i64,
    ) -> Result<Fee> {
        let pool: &PgPool = ctx.data_unchecked();
        Fee::set_amount(&name, amount, pool).await?;

        Fee::with_name(&name, pool).await
    }

    /// Charge dues for the semester
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn charge_dues(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let pool: &PgPool = ctx.data_unchecked();
        let current_semester = Semester::get_current(pool).await?;
        Fee::charge_dues_for_semester(pool).await?;

        ClubTransaction::for_semester(&current_semester.name, pool).await
    }

    /// Charges late dues for the semester (anyone who hasn't paid their due)
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn charge_late_dues(&self, ctx: &Context<'_>) -> Result<Vec<ClubTransaction>> {
        let pool: &PgPool = ctx.data_unchecked();
        let current_semester = Semester::get_current(pool).await?;
        Fee::charge_late_dues_for_semester(pool).await?;

        ClubTransaction::for_semester(&current_semester.name, pool).await
    }

    /// Creates multiple transactions from the given batch
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn add_batch_of_transactions(
        &self,
        ctx: &Context<'_>,
        batch: TransactionBatch,
    ) -> Result<Vec<ClubTransaction>> {
        let pool: &PgPool = ctx.data_unchecked();
        let current_semester = Semester::get_current(pool).await?;
        ClubTransaction::add_batch(batch, pool).await?;

        ClubTransaction::for_semester(&current_semester.name, pool).await
    }

    /// Resolves the given transaction
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_TRANSACTION)")]
    pub async fn resolve_transaction(
        &self,
        ctx: &Context<'_>,
        id: i64,
        resolved: bool,
    ) -> Result<ClubTransaction> {
        let pool: &PgPool = ctx.data_unchecked();
        ClubTransaction::resolve(id, resolved, pool).await?;

        ClubTransaction::with_id(id, pool).await
    }

    /// Sets the given global variable
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn set_variable(
        &self,
        ctx: &Context<'_>,
        key: String,
        value: String,
    ) -> Result<Variable> {
        let pool: &PgPool = ctx.data_unchecked();
        Variable::set(&key, &value, pool).await?;

        Variable::with_key(&key, pool).await
    }

    /// Unsets the given variable and returns
    #[graphql(guard = "LoggedIn.and(Permission::EDIT_OFFICERS)")]
    pub async fn unset_variable(&self, ctx: &Context<'_>, key: String) -> Result<String> {
        let pool: &PgPool = ctx.data_unchecked();
        let variable = Variable::with_key(&key, pool).await?;
        Variable::unset(&key, pool).await?;

        Ok(variable.value)
    }
}
