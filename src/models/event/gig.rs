use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::event::uniform::Uniform;
use crate::models::event::Event;
use crate::models::GqlDateTime;

#[derive(SimpleObject)]
pub struct Gig {
    /// The ID of the event this gig belongs to
    pub event: i32,
    /// When members are expected to actually perform
    pub performance_time: GqlDateTime,
    /// The name of the contact for this gig
    pub contact_name: Option<String>,
    /// The email of the contact for this gig
    pub contact_email: Option<String>,
    /// The phone number of the contact for this gig
    pub contact_phone: Option<String>,
    /// The price we are charging for this gig
    pub price: Option<i32>,
    /// Whether this gig is visible on the external website
    pub public: bool,
    /// A summary of this event for the external site (if it is public)
    pub summary: Option<String>,
    /// A description of this event for the external site (if it is public)
    pub description: Option<String>,

    #[graphql(skip)]
    pub uniform: i32,
}

#[ComplexObject]
impl Gig {
    /// The uniform for this gig
    pub async fn uniform(&self, ctx: &Context<'_>) -> Result<Uniform> {
        let conn = DbConn::from_ctx(ctx);
        Uniform::with_id(self.uniform, conn).await
    }
}

impl Gig {
    pub async fn for_event(event_id: i32, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event, performance_time as \"performance_time: _\", contact_name, contact_email,
                 contact_phone, price, public as \"public: bool\", summary, description, uniform
             FROM gig WHERE event = ?",
            event_id
        )
        .fetch_optional(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event, performance_time as \"performance_time: _\", contact_name, contact_email,
                 contact_phone, price, public as \"public: bool\", summary, description, uniform
             FROM gig WHERE event in
                 (SELECT id FROM event WHERE semester = ?)",
            semester
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum GigRequestStatus {
    Pending,
    Accepted,
    Dismissed,
}

#[derive(SimpleObject)]
pub struct GigRequest {
    /// The ID of the gig request
    pub id: i32,
    /// When the gig request was placed
    pub time: GqlDateTime,
    /// The name of the potential event
    pub name: String,
    /// The organization requesting a performance from the Glee Club
    pub organization: String,
    /// The name of the contact for the potential event
    pub contact_name: String,
    /// The email of the contact for the potential event
    pub contact_phone: String,
    /// The phone number of the contact for the potential event
    pub contact_email: String,
    /// When the event will probably happen
    pub start_time: GqlDateTime,
    /// Where the event will be happening
    pub location: String,
    /// Any comments about the event
    pub comments: Option<String>,
    /// The current status of whether the request was accepted
    pub status: GigRequestStatus,

    #[graphql(skip)]
    pub event: Option<i32>,
}

#[ComplexObject]
impl GigRequest {
    /// If and when an event is created from a request, this is the event
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Option<Event>> {
        if let Some(event_id) = self.event {
            let conn = DbConn::from_ctx(ctx);
            Ok(Some(Event::with_id(event_id, &conn).await?))
        } else {
            Ok(None)
        }
    }
}

impl GigRequest {
    pub async fn with_id(id: i32, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| format!("No gig request with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i32, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, `time` as \"time: _\", name, organization, contact_name, contact_phone, contact_email,
                 start_time as \"start_time: _\", location, comments, status as \"status: _\", event
             FROM gig_request WHERE id = ?",
            id
        )
            .fetch_optional(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, `time` as \"time: _\", name, organization, contact_name, contact_phone, contact_email,
                 start_time as \"start_time: _\", location, comments, status as \"status: _\", event
             FROM gig_request ORDER BY time"
        )
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn submit(new_request: NewGigRequest, conn: &DbConn) -> Result<i32> {
        sqlx::query!(
            "INSERT INTO gig_request (
                name, organization, contact_name, contact_phone,
                contact_email, start_time, location, comments)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            new_request.name,
            new_request.organization,
            new_request.contact_name,
            new_request.contact_phone,
            new_request.contact_email,
            new_request.start_time,
            new_request.location,
            new_request.comments
        )
        .execute(&mut *conn.get().await)
        .await?;

        sqlx::query_scalar!("SELECT id FROM gig_request ORDER BY id DESC")
            .fetch_one(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn set_status(id: i32, status: GigRequestStatus, conn: &DbConn) -> Result<()> {
        let request = Self::with_id(id, conn).await?;

        if request.status == status {
            return Ok(());
        }

        match request.status {
            GigRequestStatus::Accepted => {
                Err("Cannot change the status of an accepted gig request".into())
            }
            GigRequestStatus::Dismissed if status == GigRequestStatus::Accepted => Err(
                "Cannot directly accept a gig request if it is dismissed (please reopen it first)"
                    .into(),
            ),
            GigRequestStatus::Pending
                if status == GigRequestStatus::Accepted && request.event.is_none() =>
            {
                Err(
                    "Must create the event for the gig request first before marking it as accepted"
                        .into(),
                )
            }
            _ => {
                sqlx::query!("UPDATE gig_request SET status = ? WHERE id = ?", status, id)
                    .execute(&mut *conn.get().await)
                    .await?;

                Ok(())
            }
        }
    }

    pub async fn build_new_gig(&self, conn: &DbConn) -> Result<NewGig> {
        let default_uniform = Uniform::get_default(conn).await?;

        Ok(NewGig {
            performance_time: self.start_time.clone(),
            uniform: default_uniform.id,
            contact_name: Some(self.contact_name.clone()),
            contact_email: Some(self.contact_email.clone()),
            contact_phone: Some(self.contact_phone.clone()),
            price: Some(0),
            public: false,
            summary: None,
            description: self.comments.clone(),
        })
    }
}

#[derive(InputObject)]
pub struct NewGigRequest {
    pub name: String,
    pub organization: String,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub start_time: GqlDateTime,
    pub location: String,
    pub comments: Option<String>,
}

#[derive(InputObject)]
pub struct NewGig {
    pub performance_time: GqlDateTime,
    pub uniform: i32,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub price: Option<i32>,
    pub public: bool,
    pub summary: Option<String>,
    pub description: Option<String>,
}
