use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::models::event::Event;
use crate::models::member::Member;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Carpool {
    /// The ID of the carpool
    pub id: i64,
    /// The event it belongs to
    pub event: i64,

    #[graphql(skip)]
    pub driver: String,
}

#[ComplexObject]
impl Carpool {
    /// The driver of the carpool
    pub async fn driver(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        Member::with_email(&self.driver, &pool).await
    }

    /// The passengers of the carpool
    pub async fn passengers(&self, ctx: &Context<'_>) -> Result<Vec<Member>> {
        let pool: &PgPool = ctx.data_unchecked();
        sqlx::query_as!(
            Member,
            "SELECT email, first_name, preferred_name, last_name, phone_number, picture, passengers,
                 location, on_campus, about, major, minor, hometown,
                 arrived_at_tech, gateway_drug, conflicts, dietary_restrictions, pass_hash
             FROM members WHERE email IN
             (SELECT member FROM rides_in WHERE carpool = $1)
             ORDER BY last_name, preferred_name, first_name",
            self.id
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

impl Carpool {
    pub async fn for_event(event_id: i64, pool: &PgPool) -> Result<Vec<Carpool>> {
        sqlx::query_as!(Self, "SELECT * FROM carpools WHERE event = $1", event_id)
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        event_id: i64,
        updated_carpools: Vec<UpdatedCarpool>,
        pool: &PgPool,
    ) -> Result<()> {
        // TODO: verify exists?
        Event::with_id(event_id, pool).await?;

        sqlx::query!("DELETE FROM carpools WHERE event = $1", event_id)
            .execute(pool)
            .await?;

        // TODO: batch?
        for carpool in updated_carpools {
            sqlx::query!(
                "INSERT INTO carpools (event, driver) VALUES ($1, $2)",
                event_id,
                carpool.driver
            )
            .execute(pool)
            .await?;
            let new_carpool_id = sqlx::query_scalar!("SELECT id FROM carpools ORDER BY id DESC")
                .fetch_one(pool)
                .await?;

            for passenger in carpool.passengers {
                sqlx::query!(
                    "INSERT INTO rides_in (member, carpool) VALUES ($1, $2)",
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
