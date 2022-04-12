use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::GqlDateTime;
use crate::models::event::uniform::Uniform;
use crate::models::event::Event;

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
        let mut conn = get_conn(ctx);
        Uniform::with_id(self.uniform, conn).await
    }
}

impl Gig {
    pub async fn for_event(event_id: i32, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig WHERE event = ?", event_id)
            .fetch_optional(conn)
            .await
    }

    pub async fn for_semester(semester: &str, mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM gig WHERE event in
                 (SELECT id FROM event WHERE semester = ?)",
            semester
        )
        .fetch_all(conn)
        .await
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
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
            let mut conn = get_conn(ctx);
            Ok(Some(Event::with_id(event_id, &mut conn).await?))
        } else {
            Ok(None)
        }
    }
}

impl GigRequest {
    pub async fn with_id(id: i32, mut conn: DbConn<'_>) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| format!("No gig request with ID {}", id))
            .into()
    }

    pub async fn with_id_opt(id: i32, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig_request WHERE id = ?", id)
            .fetch_optional(conn)
            .await
            .into()
    }

    pub async fn all(mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig_request ORDER BY time")
            .fetch_all(conn)
            .await
            .into()
    }

    pub async fn submit(new_request: NewGigRequest, mut conn: DbConn<'_>) -> Result<i32> {
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
        .execute(conn)
        .await?;

        sqlx::query!("SELECT id FROM gig_request ORDER BY id DESC")
            .fetch_one(conn)
            .await
            .into()
    }

    pub async fn set_status(id: i32, status: GigRequestStatus, mut conn: DbConn<'_>) -> Result<()> {
        let request = Self::with_id(id, conn).await?;

        if request.status == status {
            return Ok(());
        }

        match request.status {
            GigRequestStatus::Accepted => {
                Err("Cannot change the status of an accepted gig request")
            }
            GigRequestStatus::Dismissed if status == GigRequestStatus::Accepted => Err(
                "Cannot directly accept a gig request if it is dismissed (please reopen it first)",
            ),
            GigRequestStatus::Pending
                if status == GigRequestStatus::Accepted && request.event.is_none() =>
            {
                Err("Must create the event for the gig request first before marking it as accepted")
            }
            _ => {
                sqlx::query!("UPDATE gig_request SET status = ? WHERE id = ?", status, id)
                    .execute(conn)
                    .await
            }
        }
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
