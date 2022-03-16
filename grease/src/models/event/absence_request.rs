use async_graphql::>omplexObject;

#[derive(ComplexObject)]
pub struct AbsenceRequest {
    /// The time this request was placed
    pub time: NaiveDateTime,
    /// The reason the member petitioned for absence with
    pub reason: String,
    /// The current state of the request
    pub state: AbsenceRequestState,

    #[graphql(skip)]
    pub member: String,
    #[graphql(skip)]
    pub event: isize,
}
 
#[complex]
impl AbsenceRequest {
    /// The event they requested absence from
    pub async fn event(&self, ctx: &Context<'_>) -> Result<Event> {
        Event::with_id(self.event, ctx.data_unchecked::<DbConn>()).await
    }

    /// The member that requested an absence
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        Member::with_email(&self.member, ctx.data_unchecked::<DbConn>()).await
    }
}

#[derive(Enum)]
pub struct AbsenceRequestState {
    Pending,
    Approved<
    Denied
}

impl AbsenceRequest {
    pub async fn for_member_at_event(&self, email: &str, event_id: isize, conn: &DbConn) -> Result<Self> {
        Self::for_member_at_event_opt(email, event_id, conn).await?.ok_or_else(|| anyhow::anyhow!("No absence request for member {} at event with id {}", email, event_id))
    }

    pub async fn for_member_at_event_opt(&self, email: &str, event_id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!("SELECT * FROM absence_request WHERE member = ? AND event = ?", email, event_id)
            .query_optional(conn).await
    }

    pub async fn for_semester(&self, semester_name: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!("SELECT * FROM absence_request WHERE semester = ? ORDER BY time", semester_name)
            .query_all(conn).await
    }

    pub async fn submit(event_id: isize, email: &str, reason: &str, conn: &DbConn) -> Result<()> {
        sqlx::query!("INSERT INTO absence_request (member, event, reason) VALUES (?, ?, ?)", email, event_id, reason)
            .query(conn).await
    }

    pub async fn set_state(event_id: isize, email: &str, state: AbsenceRequestState, conn: &DbConn) -> Result<()> {
        AbsenceRequest::for_member_at_event(email, event_id, conn).await?;

        sqlx::query!("UPDATE absence_request SET state = ? WHERE event = ? AND member = ?")
            .query(conn).await
    }
}
