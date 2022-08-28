use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::models::event::uniform::Uniform;
use crate::models::event::Event;
use crate::models::{DateTime, DateTimeInput, TimeScalar};

/// The gig info included for an event, if it is a gig
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Gig {
    /// The ID of the event this gig belongs to
    pub event: i64,
    /// The name of the contact for this gig
    pub contact_name: String,
    /// The email of the contact for this gig
    pub contact_email: String,
    /// The phone number of the contact for this gig
    pub contact_phone: String,
    /// The price we are charging for this gig
    pub price: Option<i64>,
    /// Whether this gig is visible on the external website
    pub public: bool,
    /// A summary of this event for the external site (if it is public)
    pub summary: String,
    /// A description of this event for the external site (if it is public)
    pub description: String,

    #[graphql(skip)]
    pub uniform: i64,
    #[graphql(skip)]
    pub performance_time: OffsetDateTime,
}

#[ComplexObject]
impl Gig {
    /// The uniform for this gig
    pub async fn uniform(&self, ctx: &Context<'_>) -> Result<Uniform> {
        let pool: &PgPool = ctx.data_unchecked();
        Uniform::with_id(self.uniform, pool).await
    }

    /// When members are expected to actually perform
    pub async fn performance_time(&self) -> DateTime {
        DateTime::from(self.performance_time.clone())
    }
}

impl Gig {
    pub async fn for_event(event_id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event, performance_time as \"performance_time: _\", contact_name, contact_email,
                 contact_phone, price, public as \"public: bool\", summary, description, uniform
             FROM gigs WHERE event = $1",
            event_id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT event, performance_time as \"performance_time: _\", contact_name, contact_email,
                 contact_phone, price, public as \"public: bool\", summary, description, uniform
             FROM gigs WHERE event in
                 (SELECT id FROM events WHERE semester = $1)",
            semester
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

/// The status of a gig request
#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "gig_request_status", rename_all = "snake_case")]
pub enum GigRequestStatus {
    /// We have not responded to the request yet
    Pending,
    /// We have decided to take the request
    Accepted,
    /// We have decided to decline the request
    Dismissed,
}

/// A request for the Glee Club to perform somewhere
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct GigRequest {
    /// The ID of the gig request
    pub id: i64,
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
    /// Where the event will be happening
    pub location: String,
    /// Any comments about the event
    pub comments: String,
    /// The current status of whether the request was accepted
    pub status: GigRequestStatus,

    #[graphql(skip)]
    pub event: Option<i64>,
    #[graphql(skip)]
    pub time: OffsetDateTime,
    #[graphql(skip)]
    pub start_time: OffsetDateTime,
}

#[ComplexObject]
impl GigRequest {
    /// If and when an event is created from a request, this is the event
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Option<Event>> {
        if let Some(event_id) = self.event {
            let pool: &PgPool = ctx.data_unchecked();
            Ok(Some(Event::with_id(event_id, pool).await?))
        } else {
            Ok(None)
        }
    }

    /// When the gig request was placed
    pub async fn time(&self) -> DateTime {
        DateTime::from(self.time.clone())
    }

    /// When the event will probably happen
    pub async fn start_time(&self) -> DateTime {
        DateTime::from(self.start_time.clone())
    }
}

impl GigRequest {
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No gig request with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, \"time\" as \"time: _\", name, organization, contact_name, contact_phone, contact_email,
                 start_time as \"start_time: _\", location, comments, status as \"status: _\", event
             FROM gig_requests WHERE id = $1",
            id
        )
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, \"time\" as \"time: _\", name, organization, contact_name, contact_phone, contact_email,
                 start_time as \"start_time: _\", location, comments, status as \"status: _\", event
             FROM gig_requests ORDER BY time"
        )
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn submit(new_request: NewGigRequest, pool: &PgPool) -> Result<i64> {
        sqlx::query!(
            "INSERT INTO gig_requests (
                name, organization, contact_name, contact_phone,
                contact_email, start_time, location, comments)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            new_request.name,
            new_request.organization,
            new_request.contact_name,
            new_request.contact_phone,
            new_request.contact_email,
            OffsetDateTime::from(new_request.start_time),
            new_request.location,
            new_request.comments
        )
        .execute(pool)
        .await?;

        sqlx::query_scalar!("SELECT id FROM gig_requests ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn set_status(id: i64, status: GigRequestStatus, pool: &PgPool) -> Result<()> {
        let request = Self::with_id(id, pool).await?;

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
                sqlx::query!(
                    "UPDATE gig_requests SET status = $1 WHERE id = $2",
                    status as _,
                    id
                )
                .execute(pool)
                .await?;

                Ok(())
            }
        }
    }

    pub async fn build_new_gig(&self, pool: &PgPool) -> Result<NewGig> {
        let default_uniform = Uniform::get_default(pool).await?;

        Ok(NewGig {
            performance_time: DateTimeInput::from(self.start_time).time,
            uniform: default_uniform.id,
            contact_name: self.contact_name.clone(),
            contact_email: self.contact_email.clone(),
            contact_phone: self.contact_phone.clone(),
            price: Some(0),
            public: false,
            summary: self.name.clone(),
            description: self.comments.clone(),
        })
    }
}

/// A new gig request
#[derive(InputObject)]
pub struct NewGigRequest {
    /// The name of the potential event
    pub name: String,
    /// The organization requesting a performance from the Glee Club
    pub organization: String,
    /// The name of the contact for the potential event
    pub contact_name: String,
    /// The email of the contact for the potential event
    pub contact_email: String,
    /// The phone number of the contact for the potential event
    pub contact_phone: String,
    /// When the event will probably happen
    pub start_time: DateTimeInput,
    /// Where the event will be happening
    pub location: String,
    /// Any comments about the event
    pub comments: String,
}

/// A new gig attached to a new event created from a gig request
#[derive(InputObject)]
pub struct NewGig {
    /// When we will start performing
    pub performance_time: TimeScalar,
    /// The ID of the uniform for the gig
    pub uniform: i64,
    /// The name of the contact for the gig
    pub contact_name: String,
    /// The email of the contact for the gig
    pub contact_email: String,
    /// The phone number of the contact for the gig
    pub contact_phone: String,
    /// How much we are charging for the gig
    pub price: Option<i64>,
    /// Whether we will show this gig on our external site
    pub public: bool,
    /// A title for the event on our external site
    pub summary: String,
    /// A short description for the event on our external site
    pub description: String,
}
