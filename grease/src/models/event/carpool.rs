use async_graphql::{ComplexObject, SimpleObject};
use crate::db_conn::DbConn;
use chrono::{NaiveDateTime, Local};

#[derive(SimpleObject)]
pub struct Carpool {
    /// The ID of the carpool
    pub id: isize,
    /// The event it belongs to
    pub event: isize,

    #[graphql(skip)]
    pub driver: String,
}

#[ComplexObject]
impl Carpool {
    /// The driver of the carpool
    pub async fn driver(&self, ctx: &Context<'_>) -> Result<Member> {
        Member::with_email(self.driver).await
    }

    /// The passengers of the carpool
    pub async fn passengers(&self, ctx: &Context<'_>) -> Result<Vec<Member>> {
        sqlx::query_as!(Member, "SELECT * FROM member WHERE email = (SELECT member FROM rides_in WHERE carpool = ?)", self.id).query_all(conn).await
    }
}

impl Carpool {
    pub async fn for_event(event_id: isize, conn: &DbConn) -> Result<Vec<Carpool>> {
        sqlx::query_as!(
            Self, "SELECT * FROM carpool WHERE event = ?", event_id).query_all(conn).await
    }

    pub async fn update(event_id: isize, updated_carpools: Vec<Carpool>) -> Result<()> {
        Event::verify_exists(event_id).await?;

        sqlx::query!(
            "DELETE FROM carpool WHERE event = ?", event_id
        ).query(conn).await?;

        for carpool in updated_carpools {
            sqlx::query!("INSERT INTO carpool (event, driver) VALUES (?, ?)", event_id, carpool.driver).query(conn).await?;
            let new_carpool_id: isize = sqlx::query!("SELECT id FROM carpool ORDER BY id DESC").query_one(conn).await?;

            for passenger in carpool.passengers {
                sqlx::query!("INSERT INTO rides_in (member, carpool) VALUES (?, ?)", passenger, new_id).query(conn).await?;
            }
        }

        Ok(())
    }
}

pub struct RidesIn {
    pub member: String,
    pub carpool: isize,
}
