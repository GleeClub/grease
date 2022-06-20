use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use sqlx::MySqlPool;

use crate::models::event::Event;
use crate::models::member::Member;

#[derive(SimpleObject)]
#[graphql(complex)]
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
        let pool: &MySqlPool = ctx.data_unchecked();
        Member::with_email(&self.driver, &pool).await
    }

    /// The passengers of the carpool
    pub async fn passengers(&self, ctx: &Context<'_>) -> Result<Vec<Member>> {
        let pool: &MySqlPool = ctx.data_unchecked();
        sqlx::query_as!(
            Member,
            "SELECT email, first_name, preferred_name, last_name, phone_number, picture, passengers,
                 location, on_campus as \"on_campus: bool\", about, major, minor, hometown,
                 arrived_at_tech, gateway_drug, conflicts, dietary_restrictions, pass_hash
             FROM member WHERE email IN
             (SELECT member FROM rides_in WHERE carpool = ?)
             ORDER BY last_name, preferred_name, first_name",
            self.id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

impl Carpool {
    pub async fn for_event(event_id: i32, pool: &MySqlPool) -> Result<Vec<Carpool>> {
        sqlx::query_as!(Self, "SELECT * FROM carpool WHERE event = ?", event_id)
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        event_id: i32,
        updated_carpools: Vec<UpdatedCarpool>,
        pool: &MySqlPool,
    ) -> Result<()> {
        // TODO: verify exists?
        Event::with_id(event_id, pool).await?;

        sqlx::query!("DELETE FROM carpool WHERE event = ?", event_id)
            .execute(pool)
            .await?;

        // TODO: batch?
        for carpool in updated_carpools {
            sqlx::query!(
                "INSERT INTO carpool (event, driver) VALUES (?, ?)",
                event_id,
                carpool.driver
            )
            .execute(pool)
            .await?;
            let new_carpool_id = sqlx::query_scalar!("SELECT id FROM carpool ORDER BY id DESC")
                .fetch_one(pool)
                .await?;

            for passenger in carpool.passengers {
                sqlx::query!(
                    "INSERT INTO rides_in (member, carpool) VALUES (?, ?)",
                    passenger,
                    new_carpool_id
                )
                .execute(pool)
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
