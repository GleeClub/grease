use db::schema::{carpool, member, rides_in};
use db::{Carpool, Member, NewCarpool, RidesIn, UpdatedCarpool};
use diesel::prelude::*;
use error::*;
use serde_json::{json, Value};

impl Carpool {
    pub fn load_for_event(
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventCarpool>> {
        use db::schema::carpool::dsl::{event, id};

        let carpool_driver_pairs = carpool::table
            .inner_join(member::table)
            .filter(event.eq(given_event_id))
            .order_by(id.asc())
            .load(conn)
            .map_err(GreaseError::DbError)?;

        let passenger_pairs = rides_in::table
            .inner_join(member::table)
            .filter(
                rides_in::dsl::carpool
                    .eq_any(carpool::table.select(id).filter(event.eq(given_event_id))),
            )
            .load::<(RidesIn, Member)>(conn)
            .map_err(GreaseError::DbError)?;

        let mut carpools = carpool_driver_pairs
            .into_iter()
            .map(|(found_carpool, found_driver)| EventCarpool {
                driver: found_driver,
                carpool: found_carpool,
                passengers: Vec::new(),
            })
            .collect::<Vec<EventCarpool>>();
        for (passenger_rides_in, passenger) in passenger_pairs {
            carpools
                .iter_mut()
                .find(|some_carpool| some_carpool.carpool.id == passenger_rides_in.carpool)
                .map(|found_carpool| found_carpool.passengers.push(passenger));
        }

        Ok(carpools)
    }

    pub fn update_for_event(
        given_event_id: i32,
        new_carpools: Vec<UpdatedCarpool>,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::carpool::dsl::{carpool, event, id};

        conn.transaction(|| {
            diesel::delete(carpool.filter(event.eq(given_event_id)))
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            diesel::insert_into(carpool)
                .values(
                    &new_carpools
                        .iter()
                        .map(|c| NewCarpool {
                            event: given_event_id,
                            driver: c.driver.clone(),
                        })
                        .collect::<Vec<NewCarpool>>(),
                )
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            let new_carpool_ids: Vec<i32> = carpool
                .select(id)
                .order_by(id.desc())
                .limit(new_carpools.len() as i64)
                .load(conn)
                .map_err(GreaseError::DbError)?;

            let rides_ins = new_carpool_ids
                .into_iter()
                .zip(new_carpools.into_iter())
                .flat_map(|(new_id, new_carpool)| {
                    new_carpool
                        .passengers
                        .into_iter()
                        .map(move |passenger| RidesIn {
                            carpool: new_id,
                            member: passenger,
                        })
                })
                .collect::<Vec<RidesIn>>();

            diesel::insert_into(rides_in::table)
                .values(&rides_ins)
                .execute(conn)
                .map_err(GreaseError::DbError)?;

            Ok(())
        })
    }
}

pub struct EventCarpool {
    pub driver: Member,
    pub carpool: Carpool,
    pub passengers: Vec<Member>,
}

impl EventCarpool {
    /// Render this event's carpool data to JSON.
    ///
    /// ## JSON Format:
    ///
    /// ```json
    /// {
    ///     "id": integer,
    ///     "event": integer,
    ///     "driver": Member,
    ///     "passengers": [Member]
    /// }
    /// ```
    ///
    /// See [Member](../struct.Member.html#json-format) and
    /// [Carpool](../struct.Carpool.html#json-format) for their
    /// respective JSON formats.
    pub fn to_json(&self) -> Value {
        json!({
            "driver": self.driver,
            "id": self.carpool.id,
            "event": self.carpool.event,
            "passengers": self.passengers,
        })
    }
}
