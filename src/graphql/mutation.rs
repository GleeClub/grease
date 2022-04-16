use async_graphql::{Context, Object, Result};

use crate::db::DbConn;
use crate::graphql::permission::Permission;
use crate::graphql::{LoggedIn, SUCCESS_MESSAGE};
use crate::models::member::active_semester::ActiveSemester;
use crate::models::event::attendance::{Attendance, AttendanceUpdate};
use crate::models::event::{Event, NewEvent};
use crate::models::member::session::Session;
use crate::models::member::{Member, MemberUpdate, NewMember, RegisterForSemesterForm};

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

    #[graphql(guard = "LoggedIn.and((Permission::CREATE_EVENT.for_type(&new_event.event.r#type)))")]
    pub async fn create_event(&self, ctx: &Context<'_>, new_event: NewEvent) -> Result<Event> {
        let conn = DbConn::from_ctx(ctx);
        let new_id = Event::create(new_event, None, conn).await?;

        Event::with_id(new_id, conn).await
    }

    #[graphql(guard = "LoggedIn.and((Permission::MODIFY_EVENT.for_type(&new_event.event.r#type)))")]
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
    /// Deletes an event and returns its id
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

        if !Permission::EDIT_ATTENDANCE.for_type(&event.r#type).granted_to(&user.email, conn).await? {
            let user_section = ActiveSemester::for_member_during_semester(&user.email, &event.semester, conn).await?
                .map(|semester| semester.section);
            let member_section = ActiveSemester::for_member_during_semester(&email, &event.semester, conn).await?
                .map(|semester| semester.section);

            if user_section.is_none() || user_section != member_section || !Permission::EDIT_ATTENDANCE_OWN_SECTION.for_type(&event.r#type).granted_to(&user.email, conn).await? {
                return Err("Not allowed to edit attendance".into());
            }
        }

        Attendance::update(event_id, &email, update, conn).await?;

        Attendance::for_member_at_event(&email, event_id, conn).await
    }
}

//   @[GraphQL::Field]
//   def rsvp_for_event(id : Int32, attending : Bool, context : UserContext) : Models::Attendance
//     Models::Attendance.rsvp_for_event id, context.user!, attending
//     Models::Attendance.for_member_at_event! context.user!.email, id
//   end

//   @[GraphQL::Field]
//   def confirm_for_event(id : Int32, context : UserContext) : Models::Attendance
//     Models::Attendance.confirm_for_event id, context.user!
//     Models::Attendance.for_member_at_event! context.user!.email, id
//   end

//   @[GraphQL::Field]
//   def update_carpools(event_id : Int32, carpools : Array(Input::UpdatedCarpool), context : UserContext) : Array(Models::Carpool)
//     context.able_to! Permissions::EDIT_CARPOOLS

//     Models::Carpool.update event_id, carpools
//     Models::Carpool.for_event event_id
//   end

//   @[GraphQL::Field]
//   def respond_to_absence_request(event_id : Int32, member : String, approved : Bool, context : UserContext) : Models::AbsenceRequest
//     context.able_to! Permissions::PROCESS_ABSENCE_REQUESTS

//     state = approved ? Models::AbsenceRequest::State::APPROVED : Models::AbsenceRequest::State::DENIED
//     Models::AbsenceRequest.set_state event_id, member, state
//     Models::AbsenceRequest.for_member_at_event! member, event_id
//   end

//   @[GraphQL::Field]
//   def submit_absence_request(event_id : Int32, reason : String, context : UserContext) : Models::AbsenceRequest
//     Models::AbsenceRequest.submit event_id, context.user!.email, reason
//     Models::AbsenceRequest.for_member_at_event! event_id, context.user!.email
//   end

//   @[GraphQL::Field]
//   def submit_gig_request(form : Input::NewGigRequest) : Models::GigRequest
//     new_id = Models::GigRequest.submit form
//     Models::GigRequest.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def dismiss_gig_request(id : Int32, context : UserContext) : Models::GigRequest
//     context.able_to! Permissions::PROCESS_GIG_REQUESTS

//     request = Models::GigRequest.with_id! id
//     request.set_status Models::GigRequest::Status::DISMISSED
//     request
//   end

//   @[GraphQL::Field]
//   def reopen_gig_request(id : Int32, context : UserContext) : Models::GigRequest
//     context.able_to! Permissions::PROCESS_GIG_REQUESTS

//     request = Models::GigRequest.with_id! id
//     request.set_status Models::GigRequest::Status::PENDING
//     request
//   end

//   @[GraphQL::Field]
//   def create_event_from_gig_request(request_id : Int32, form : Input::NewEvent, context : UserContext) : Models::Event
//     context.able_to! Permissions::CREATE_EVENT, form.event.type

//     request = Models::GigRequest.with_id request_id
//     new_id = Models::Event.create form, request
//     Models::Event.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def set_variable(key : String, value : String, context : UserContext) : Models::Variable
//     context.able_to! Permissions::EDIT_OFFICERS

//     Models::Variable.set key, value
//     Models::Variable.with_key! key
//   end

//   @[GraphQL::Field]
//   def unset_variable(key : String, context : UserContext) : Models::Variable
//     context.able_to! Permissions::EDIT_OFFICERS

//     var = Models::Variable.with_key! key
//     var.unset
//     var
//   end

//   @[GraphQL::Field]
//   def create_document(name : String, url : String, context : UserContext) : Models::Document
//     context.able_to! Permissions::EDIT_LINKS

//     Models::Document.create name, url
//     Models::Document.with_name! name
//   end

//   @[GraphQL::Field]
//   def update_document(name : String, url : String, context : UserContext) : Models::Document
//     context.able_to! Permissions::EDIT_LINKS

//     document = Models::Document.with_name! name
//     document.set_url url
//     document
//   end

//   @[GraphQL::Field]
//   def delete_document(name : String, context : UserContext) : Models::Document
//     context.able_to! Permissions::EDIT_LINKS

//     document = Models::Document.with_name! name
//     document.delete
//     document
//   end

//   @[GraphQL::Field]
//   def create_semester(form : Input::NewSemester, context : UserContext) : Models::Semester
//     context.able_to! Permissions::EDIT_SEMESTER

//     Models::Semester.create form
//     Models::Semester.with_name! form.name
//   end

//   @[GraphQL::Field]
//   def update_semester(name : String, form : Input::NewSemester, context : UserContext) : Models::Semester
//     context.able_to! Permissions::EDIT_SEMESTER

//     Models::Semester.update name, form
//     Models::Semester.with_name! form.name
//   end

//   @[GraphQL::Field]
//   def set_current_semester(name : String, context : UserContext) : Models::Semester
//     context.able_to! Permissions::EDIT_SEMESTER

//     Models::Semester.set_current name
//     Models::Semester.with_name! name
//   end

//   @[GraphQL::Field]
//   def create_meeting_minutes(name : String, context : UserContext) : Models::Minutes
//     context.able_to! Permissions::EDIT_MINUTES

//     new_id = Models::Minutes.create name
//     Models::Minutes.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def update_meeting_minutes(id : Int32, form : Input::UpdatedMeetingMinutes, context : UserContext) : Models::Minutes
//     context.able_to! Permissions::EDIT_MINUTES

//     minutes = Models::Minutes.with_id! id
//     minutes.update form
//     minutes
//   end

//   @[GraphQL::Field]
//   def email_meeting_minutes(id : Int32, context : UserContext) : Models::Minutes
//     context.able_to! Permissions::EDIT_MINUTES

//     minutes = Models::Minutes.with_id! id
//     minutes.email
//     minutes
//   end

//   @[GraphQL::Field]
//   def delete_meeting_minutes(id : Int32, context : UserContext) : Models::Minutes
//     context.able_to! Permissions::EDIT_MINUTES

//     minutes = Models::Minutes.with_id! id
//     minutes.delete
//     minutes
//   end

//   @[GraphQL::Field]
//   def create_uniform(form : Input::NewUniform, context : UserContext) : Models::Uniform
//     context.able_to! Permissions::EDIT_UNIFORMS

//     new_id = Models::Uniform.create form
//     Models::Uniform.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def update_uniform(id : Int32, form : Input::NewUniform, context : UserContext) : Models::Uniform
//     context.able_to! Permissions::EDIT_UNIFORMS

//     uniform = Models::Uniform.with_id! id
//     uniform.update form
//     uniform
//   end

//   @[GraphQL::Field]
//   def delete_uniform(id : Int32, context : UserContext) : Models::Uniform
//     context.able_to! Permissions::EDIT_UNIFORMS

//     uniform = Models::Uniform.with_id! id
//     uniform.delete
//     uniform
//   end

//   @[GraphQL::Field]
//   def create_song(form : Input::NewSong, context : UserContext) : Models::Song
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     new_id = Models::Song.create form
//     Models::Song.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def update_song(id : Int32, form : Input::SongUpdate, context : UserContext) : Models::Song
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     song = Models::Song.with_id! id
//     song.update form
//     song
//   end

//   @[GraphQL::Field(description: "Deletes a song and returns the id")]
//   def delete_song(id : Int32, context : UserContext) : Int32
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     song = Models::Song.with_id! id
//     song.delete
//     id
//   end

//   @[GraphQL::Field]
//   def create_song_link(song_id : Int32, form : Input::NewSongLink, context : UserContext) : Models::SongLink
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     new_id = Models::SongLink.create song_id, form
//     Models::SongLink.with_id! new_id
//   end

//   @[GraphQL::Field]
//   def update_song_link(id : Int32, form : Input::SongLinkUpdate, context : UserContext) : Models::SongLink
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     link = Models::SongLink.with_id! id
//     link.update form
//     link
//   end

//   @[GraphQL::Field]
//   def delete_song_link(id : Int32, context : UserContext) : Models::SongLink
//     context.able_to! Permissions::EDIT_REPERTOIRE

//     link = Models::SongLink.with_id! id
//     link.delete
//     link
//   end

//   @[GraphQL::Field]
//   def add_permission_to_role(position : String, permission : String, event_type : String?, context : UserContext) : Bool
//     context.able_to! Permissions::EDIT_PERMISSIONS

//     Models::RolePermission.add position, permission, event_type
//     true
//   end

//   @[GraphQL::Field]
//   def remove_permission_from_role(position : String, permission : String, event_type : String?, context : UserContext) : Bool
//     context.able_to! Permissions::EDIT_PERMISSIONS

//     Models::RolePermission.remove position, permission, event_type
//     true
//   end

//   @[GraphQL::Field]
//   def add_officership(position : String, member : String, context : UserContext) : Models::MemberRole
//     context.able_to! Permissions::EDIT_OFFICERS

//     member_role = Models::MemberRole.new member, position
//     member_role.add
//     member_role
//   end

//   @[GraphQL::Field]
//   def remove_officership(position : String, member : String, context : UserContext) : Models::MemberRole
//     context.able_to! Permissions::EDIT_OFFICERS

//     member_role = Models::MemberRole.new member, position
//     member_role.remove
//     member_role
//   end

//   @[GraphQL::Field]
//   def update_fee_amount(name : String, amount : Int32, context : UserContext) : Models::Fee
//     context.able_to! Permissions::EDIT_TRANSACTION

//     fee = Models::Fee.with_name! name
//     fee.set_amount amount
//     fee
//   end

//   @[GraphQL::Field]
//   def charge_dues(context : UserContext) : Array(Models::ClubTransaction)
//     context.able_to! Permissions::EDIT_TRANSACTION

//     Models::Fee.charge_dues_for_semester
//     Models::ClubTransaction.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def charge_late_dues(context : UserContext) : Array(Models::ClubTransaction)
//     context.able_to! Permissions::EDIT_TRANSACTION

//     Models::Fee.charge_late_dues_for_semester
//     Models::ClubTransaction.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def add_batch_of_transactions(batch : Input::TransactionBatch, context : UserContext) : Array(Models::ClubTransaction)
//     context.able_to! Permissions::EDIT_TRANSACTION

//     Models::ClubTransaction.add_batch batch
//     Models::ClubTransaction.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def resolve_transaction(id : Int32, resolved : Bool, context : UserContext) : Models::ClubTransaction
//     context.able_to! Permissions::EDIT_TRANSACTION

//     transaction = Models::ClubTransaction.with_id! id
//     transaction.resolve resolved
//     transaction
//   end

//   # TODO: sendEmail(since: NaiveDateTime!): Boolean!
// end
