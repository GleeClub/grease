use db::models::*;
use db::schema::carpool::dsl::*;
use db::schema::{carpool, member};
use diesel::mysql::MysqlConnection;
use diesel::*;
use error::*;
use extract_derive::Extract;
use serde::Deserialize;

impl Carpool {
    pub fn load(given_carpool_id: i32, conn: &MysqlConnection) -> GreaseResult<Carpool> {
        carpool
            .filter(id.eq(given_carpool_id))
            .first::<Carpool>(conn)
            .optional()
            .map_err(GreaseError::DbError)?
            .ok_or(GreaseError::BadRequest(format!(
                "carpool with id {} doesn't exist",
                given_carpool_id
            )))
    }

    pub fn load_for_event(
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<EventCarpool>> {
        let (carpool_driver_pairs, passenger_pairs) = conn
            .transaction(|| {
                let carpool_driver_pairs = carpool
                    .inner_join(member::table)
                    .filter(event.eq(given_event_id))
                    .order(id)
                    .load::<(Carpool, Member)>(conn)?;

                let passenger_pairs = rides_in::table
                    .inner_join(member::table)
                    .filter(
                        rides_in::dsl::carpool.eq_any(
                            carpool
                                .filter(event.eq(given_event_id))
                                .select(id)
                                .load::<i32>(conn)?,
                        ),
                    )
                    .load::<(RidesIn, Member)>(conn)?;

                Ok((carpool_driver_pairs, passenger_pairs))
            })
            .map_err(GreaseError::DbError)?;

        let mut carpools = carpool_driver_pairs
            .into_iter()
            .map(|(found_carpool, found_driver)| EventCarpool {
                driver: found_driver,
                carpool: found_carpool,
                passengers: Vec::new(),
            })
            .collect::<Vec<EventCarpool>>();

        for (rides_in, passenger) in passenger_pairs {
            carpools
                .iter_mut()
                .find(|some_carpool| some_carpool.carpool.event == rides_in.carpool)
                .map(|found_carpool| found_carpool.passengers.push(passenger));
        }

        Ok(carpools)
    }

    // TODO: figure out what to do with actual carpool uploading / creation
    pub fn create(new_carpool: NewCarpool, conn: &MysqlConnection) -> GreaseResult<i32> {
        conn.transaction(|| {
            diesel::insert_into(carpool)
                .values(&new_carpool)
                .execute(conn)?;

            carpool.order(id.desc()).select(id).first(conn)
        })
        .map_err(GreaseError::DbError)
    }

    pub fn create_multiple(
        new_carpools: Vec<NewCarpool>,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        diesel::insert_into(carpool)
            .values(&new_carpools)
            .execute(conn)
            .map_err(GreaseError::DbError)?;
        Ok(())
    }

    pub fn update_for_event(
        given_event_id: i32,
        mut updated_carpools: Vec<UpdatedCarpool>,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        let (new_carpools, all_new_passengers): (Vec<NewCarpool>, Vec<Vec<String>>) =
            updated_carpools
                .drain_filter(|updated| updated.id.is_none())
                .map(|updated| {
                    let new_carpool = NewCarpool {
                        event: given_event_id,
                        driver: updated.driver,
                    };
                    (new_carpool, updated.passengers)
                })
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut new_carpools, mut all_new_passengers), (new_carpool, passengers)| {
                        new_carpools.push(new_carpool);
                        all_new_passengers.push(passengers);
                        (new_carpools, all_new_passengers)
                    },
                );

        let (updated_carpools, updated_passengers): (Vec<Carpool>, Vec<Vec<String>>) =
            updated_carpools
                .into_iter()
                .map(|updated| {
                    let updated_carpool = Carpool {
                        id: updated.id.unwrap(),
                        event: given_event_id,
                        driver: updated.driver,
                    };
                    (updated_carpool, updated.passengers)
                })
                .fold(
                    (Vec::new(), Vec::new()),
                    |(mut updated_carpools, mut updated_passengers),
                     (updated_carpool, passengers)| {
                        updated_carpools.push(updated_carpool);
                        updated_passengers.push(passengers);
                        (updated_carpools, updated_passengers)
                    },
                );

        conn.transaction(|| {
            let old_carpool_ids = carpool
                .filter(event.eq(given_event_id))
                .select(id)
                .load::<i32>(conn)?;

            diesel::delete(rides_in::table.filter(rides_in::dsl::carpool.eq_any(&old_carpool_ids)))
                .execute(conn)?;

            diesel::insert_into(carpool)
                .values(&new_carpools)
                .execute(conn)?;

            let new_carpool_ids = carpool
                .order(id.desc())
                .select(id)
                .limit(new_carpools.len() as i64)
                .load::<i32>(conn)?;

            for updated_carpool in updated_carpools.iter() {
                diesel::update(carpool.filter(id.eq(&updated_carpool.id)))
                    .set((
                        event.eq(&updated_carpool.event),
                        driver.eq(&updated_carpool.driver),
                    ))
                    .execute(conn)?;
            }

            let updated_rides_ins = new_carpool_ids
                .into_iter()
                .rev()
                .chain(
                    updated_carpools
                        .into_iter()
                        .map(|updated_carpool| updated_carpool.id),
                )
                .zip(
                    all_new_passengers
                        .into_iter()
                        .chain(updated_passengers.into_iter()),
                )
                .flat_map(|(new_id, new_passengers)| {
                    new_passengers
                        .into_iter()
                        .map(move |new_passenger| RidesIn {
                            member: new_passenger,
                            carpool: new_id,
                        })
                })
                .collect::<Vec<RidesIn>>();

            diesel::insert_into(rides_in::table)
                .values(&updated_rides_ins)
                .execute(conn)?;

            Ok(())
        })
        .map_err(GreaseError::DbError)?;

        Ok(())
    }
}

// TODO: figure out group_by for this puppy
#[derive(Debug, Deserialize)]
pub struct EventCarpool {
    pub driver: Member,
    pub carpool: Carpool,
    pub passengers: Vec<Member>,
}

#[derive(Debug, Deserialize, Extract)]
pub struct UpdatedCarpool {
    pub id: Option<i32>,
    pub driver: String,
    pub passengers: Vec<String>,
}

#[derive(Deserialize, AsChangeset, Insertable, Extract)]
#[table_name = "carpool"]
pub struct NewCarpool {
    pub event: i32,
    pub driver: String,
}
