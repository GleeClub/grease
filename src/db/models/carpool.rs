use db::models::*;
use db::traits::*;
use error::*;
use mysql::Conn;
use pinto::query_builder::{self, Join, Order};
use serde::Deserialize;
use serde_json::{json, Value};

impl Carpool {
    pub fn load_for_event(given_event_id: i32, conn: &mut Conn) -> GreaseResult<Vec<EventCarpool>> {
        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;

        let query = query_builder::select(Carpool::table_name())
            .join(Member::table_name(), "member", "email", Join::Inner)
            .fields(CarpoolMemberRow::field_names())
            .filter(&format!("event = {}", given_event_id))
            .order_by("id", Order::Asc)
            .build();
        let carpool_driver_pairs = crate::db::load::<CarpoolMemberRow, _>(&query, &mut transaction)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())?;

        let inner_query = query_builder::select(RidesIn::table_name())
            .fields(&["id"])
            .filter(&format!("event = {}", given_event_id))
            .build();
        let query = query_builder::select(RidesIn::table_name())
            .join(Member::table_name(), "member", "email", Join::Inner)
            .fields(RidesInMemberRow::field_names())
            .filter(&format!("event = ({})", inner_query))
            .build();
        let passenger_pairs = crate::db::load::<RidesInMemberRow, _>(&query, &mut transaction)
            .map(|rows| rows.into_iter().map(|row| row.into()).collect::<Vec<_>>())?;

        transaction.commit().map_err(GreaseError::DbError)?;

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

    pub fn update_for_event(
        given_event_id: i32,
        mut updated_carpools: Vec<UpdatedCarpool>,
        conn: &mut Conn,
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

        let mut transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;

        let inner_ids_query = query_builder::select(Carpool::table_name())
            .fields(&["id"])
            .build();
        let delete_query = query_builder::delete(RidesIn::table_name())
            .filter(&format!("carpool = ({})", inner_ids_query))
            .build();
        transaction
            .query(delete_query)
            .map_err(GreaseError::DbError)?;

        new_carpools
            .iter()
            .map(|new_carpool| new_carpool.insert(&mut transaction))
            .collect::<GreaseResult<()>>()?;
        let new_carpool_id_query = query_builder::select(Carpool::table_name())
            .fields(&["id"])
            .order_by("id", Order::Desc)
            .limit(new_carpools.len())
            .build();
        let new_carpool_ids = crate::db::load::<i32, _>(&new_carpool_id_query, &mut transaction)?;

        updated_carpools
            .iter()
            .map(|updated_carpool| {
                let update_query = query_builder::update(Carpool::table_name())
                    .filter(&format!("id = {}", updated_carpool.id))
                    .set("event", &updated_carpool.event.to_string())
                    .set("driver", &format!("'{}'", &updated_carpool.driver))
                    .build();

                transaction
                    .query(update_query)
                    .map_err(GreaseError::DbError)?;
                Ok(())
            })
            .collect::<GreaseResult<()>>()?;

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

        updated_rides_ins
            .into_iter()
            .map(|updated_rides_in| updated_rides_in.insert(&mut transaction))
            .collect::<GreaseResult<()>>()?;
        transaction.commit().map_err(GreaseError::DbError)?;

        Ok(())
    }
}

// TODO: figure out group_by for this puppy
#[derive(Deserialize, Serialize)]
pub struct EventCarpool {
    pub driver: Member,
    pub carpool: Carpool,
    pub passengers: Vec<Member>,
}

impl EventCarpool {
    pub fn to_json(&self) -> Value {
        json!({
            "driver": self.driver.to_json(),
            "carpool": self.carpool,
            "passengers": self.passengers.iter()
                .map(|passenger| passenger.to_json())
                .collect::<Vec<_>>()
        })
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
pub struct CarpoolMemberRow {
    // carpool fields
    pub id: i32,
    pub event: i32,
    pub driver: String,
    // member fields
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
}

impl Into<(Carpool, Member)> for CarpoolMemberRow {
    fn into(self) -> (Carpool, Member) {
        (
            Carpool {
                id: self.id,
                event: self.event,
                driver: self.driver,
            },
            Member {
                email: self.email,
                first_name: self.first_name,
                preferred_name: self.preferred_name,
                last_name: self.last_name,
                pass_hash: self.pass_hash,
                phone_number: self.phone_number,
                picture: self.picture,
                passengers: self.passengers,
                location: self.location,
                about: self.about,
                major: self.major,
                minor: self.minor,
                hometown: self.hometown,
                arrived_at_tech: self.arrived_at_tech,
                gateway_drug: self.gateway_drug,
                conflicts: self.conflicts,
                dietary_restrictions: self.dietary_restrictions,
            },
        )
    }
}

#[derive(grease_derive::FromRow, grease_derive::FieldNames)]
pub struct RidesInMemberRow {
    // rides in fields
    pub member: String,
    pub carpool: i32,
    // member fields
    pub email: String,
    pub first_name: String,
    pub preferred_name: Option<String>,
    pub last_name: String,
    pub pass_hash: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub passengers: i32,
    pub location: String,
    pub about: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub hometown: Option<String>,
    pub arrived_at_tech: Option<i32>,
    pub gateway_drug: Option<String>,
    pub conflicts: Option<String>,
    pub dietary_restrictions: Option<String>,
}

impl Into<(RidesIn, Member)> for RidesInMemberRow {
    fn into(self) -> (RidesIn, Member) {
        (
            RidesIn {
                member: self.member,
                carpool: self.carpool,
            },
            Member {
                email: self.email,
                first_name: self.first_name,
                preferred_name: self.preferred_name,
                last_name: self.last_name,
                pass_hash: self.pass_hash,
                phone_number: self.phone_number,
                picture: self.picture,
                passengers: self.passengers,
                location: self.location,
                about: self.about,
                major: self.major,
                minor: self.minor,
                hometown: self.hometown,
                arrived_at_tech: self.arrived_at_tech,
                gateway_drug: self.gateway_drug,
                conflicts: self.conflicts,
                dietary_restrictions: self.dietary_restrictions,
            },
        )
    }
}
