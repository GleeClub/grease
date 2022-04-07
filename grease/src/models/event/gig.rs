use async_graphql::{ComplexObject, InputObject, SimpleObject, Enum};

pub struct Gig {
    /// The ID of the event this gig belongs to
    pub event: isize,
    /// When members are expected to actually perform
    pub performance_time: NaiveDateTime,
    /// The name of the contact for this gig
    pub contact_name: Option<String>,
    /// The email of the contact for this gig
    pub contact_email: Option<String>,
    /// The phone number of the contact for this gig
    pub contact_phone: Option<String>,
    /// The price we are charging for this gig
    pub price: Option<isize>,
    /// Whether this gig is visible on the external website
    pub public: bool,
    /// A summary of this event for the external site (if it is public)
    pub summary: Option<String>,
    /// A description of this event for the external site (if it is public)
    pub description: Option<String>,

    #[graphql(skip)]
    pub uniform: isize,
}

#[ComplexObject]
impl Gig {
    /// The uniform for this gig
    pub async fn uniform(&self, ctx: &Context<'_>) -> Result<Uniform> {
        let conn = ctx.data_unchecked::<DbConn>();
        Uniform::with_id(self.uniform, conn).await
    }
}

impl Gig {
    pub fn for_event(event_id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig WHERE event = ?", event_id).query_optional(conn).await
    }
}

#[derive(Enum)]
pub enum GigRequestStatus {
    Pending,
    Accepted,
    Dismissed,
}

pub struct GigRequest {
    /// The ID of the gig request
    pub id: isize,
    /// When the gig request was placed
    pub time: NaiveDateTime,
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
    pub start_time: NaiveDateTime,
    /// Where the event will be happening
    pub location: String,
    /// Any comments about the event
    pub comments: Option<String>,
    /// The current status of whether the request was accepted
    pub status: GigRequestStatus,

    #[graphql(skip)]
    pub event: Option<isize>,
}

#[ComplexObject]
impl GigRequest {
    /// If and when an event is created from a request, this is the event
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Option<Event>> {
        if let Some(event_id) = self.event {
            let conn = ctx.data_unchecked::<DbConn>();
            Ok(Some(Event::with_id(event_id).await?))
        } else {
            Ok(None)
        }
    }
}

impl GigRequest {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| format!("No gig request with ID {}", id))
    }

    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig_request WHERE id = ?", id).query_optional(conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM gig_request ORDER BY time").query_all(conn).await
    }

    pub async fn submit(new_request: NewGigRequest,conn: &DbConn) -> Result<isize> {
        sqlx::query!(
            "INSERT INTO gig_request (
                name, organization, contact_name, contact_phone,
                contact_email, start_time, location, comments)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
             new_request.name, new_request.organization, new_request.contact_name, new_request.contact_phone,
             new_request.contact_email, new_request.start_time, new_request.location, new_request.comments
        ).query(conn).await?;

        sqlx::query!("SELECT id FROM gig_request ORDER BY id DESC").query_one(conn).await
    }

    pub async fn set_status(id: isize, status: GigRequestStatus, conn: &DbConn) -> Result<()> {
        let request = Self::with_id(id, conn).await?;

        if request.status == status {
            return Ok(());
        }

        match request.status {
            GigRequestStatus::Accepted => {
                Err("Cannot change the status of an accepted gig request")
            }
            GigRequestStatus::Dismissed if status == GigRequestStatus::Accepted => {
                Err("Cannot directly accept a gig request if it is dismissed (please reopen it first)")
            }
            GigRequestStatus::Pending if status == GigRequestStatus::Accepted && self.event.is_none() => {
                Err("Must create the event for the gig request first before marking it as accepted")
            }
            _ => {
                sqlx::query!("UPDATE gig_request SET status = ? WHERE id = ?", status, id).query(conn).await
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
    pub start_time: NaiveDateTime,
    pub location: String,
    pub comments: Option<String>,
}
