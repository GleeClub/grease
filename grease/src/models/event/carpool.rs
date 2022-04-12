use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, Result};

use crate::db::DbConn;
use crate::models::event::Event;
use crate::models::member::Member;

#[derive(SimpleObject)]
pub struct Carpool {
    /// The ID of the carpool
    pub id: i32,
    /// The event it belongs to
    pub event: i32,

    #[graphql(skip)]
    pub driver: String,
}

#[ComplexObject]
impl Carpool {
    /// The driver of the carpool
    pub async fn driver(&self, ctx: &Context<'_>) -> Result<Member> {
        let mut conn = get_conn(ctx);
        Member::with_email(&self.driver, &mut conn).await
    }

    /// The passengers of the carpool
    pub async fn passengers(&self, ctx: &Context<'_>) -> Result<Vec<Member>> {
        let mut conn = get_conn(ctx);
        sqlx::query_as!(
            Member,
            "SELECT * FROM member WHERE email IN
             (SELECT member FROM rides_in WHERE carpool = ?)
             ORDER BY last_name, first_name",
            self.id
        )
        .fetch_all(&mut conn)
        .await
    }
}

impl Carpool {
    pub async fn for_event(event_id: i32, mut conn: DbConn<'_>) -> Result<Vec<Carpool>> {
        sqlx::query_as!(Self, "SELECT * FROM carpool WHERE event = ?", event_id)
            .fetch_all(conn)
            .await
    }

    pub async fn update(
        event_id: i32,
        updated_carpools: Vec<Carpool>,
        mut conn: DbConn<'_>,
    ) -> Result<()> {
        Event::verify_exists(event_id).await?;

        sqlx::query!("DELETE FROM carpool WHERE event = ?", event_id)
            .execute(conn)
            .await?;

        // TODO: batch?
        for carpool in updated_carpools {
            sqlx::query!(
                "INSERT INTO carpool (event, driver) VALUES (?, ?)",
                event_id,
                carpool.driver
            )
            .execute(conn)
            .await?;
            let new_carpool_id: i32 = sqlx::query!("SELECT id FROM carpool ORDER BY id DESC")
                .fetch_one(conn)
                .await?;

            for passenger in carpool.passengers {
                sqlx::query!(
                    "INSERT INTO rides_in (member, carpool) VALUES (?, ?)",
                    passenger,
                    new_carpool_id
                )
                .execute(conn)
                .await?;
            }
        }

        Ok(())
    }
}

#[derive(InputObject)]
pub struct UpdatedCarpool {
    pub driver: String,
    pub passengers: Vec<String>,
}
