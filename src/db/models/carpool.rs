use db::models::*;
use db::schema::carpool::dsl::*;
use db::schema::member::dsl::{first_name, last_name};
use db::schema::{carpool, member};
use diesel::mysql::MysqlConnection;
use diesel::result::QueryResult;
use diesel::*;

impl Carpool {
    pub fn load(given_carpool_id: i32, conn: &MysqlConnection) -> Result<Carpool, String> {
        carpool
            .filter(id.eq(given_carpool_id))
            .first::<Carpool>(conn)
            .optional()
            .expect("error loading carpool")
            .ok_or(format!(
                "carpool with id {} doesn't exist",
                given_carpool_id
            ))
    }

    pub fn load_for_event(
        given_event_id: i32,
        conn: &MysqlConnection,
    ) -> Result<EventCarpools, String> {
        let event = Event::load(given_event_id, conn)?;
        let mut carpool_user_pairs = carpool::table
            .inner_join(member::table)
            .filter(event.eq(given_event_id))
            .order(first_name) // TODO: pick order here
            .order(last_name)
            .load::<(Carpool, User)>(conn)
            .expect("error loading carpools");

        let driver_pairs = carpool_user_pairs
            .drain_filter(|(c, _u)| c.is_driver)
            .collect::<Vec<_>>();

        let event_carpools = driver_pairs
            .into_iter()
            .map(move |(carpool, user)| EventCarpool {
                driver: user,
                carpool,
                passengers: carpool_user_pairs
                    .drain_filter(|(c, u)| {
                        if let Some(ref d_email) = c.driver_email {
                            d_email == &u.email
                        } else {
                            false
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(EventCarpools {
            event,
            carpools: event_carpools,
        })
    }

    // TODO: figure out what to do with actual carpool uploading / creation
    pub fn create(new_carpool: NewCarpool, conn: &MysqlConnection) {
        diesel::insert_into(carpool)
            .values(&new_carpool)
            .execute(conn)
            .expect("error adding new carpool");
    }

    pub fn create_multiple(new_carpools: Vec<NewCarpool>, conn: &MysqlConnection) {
        diesel::insert_into(carpool)
            .values(&new_carpools)
            .execute(conn)
            .expect("error adding new carpools");
    }

    pub fn update(given_carpool_id: i32, updated_carpool: NewCarpool, conn: &MysqlConnection) -> bool {
        diesel::update(carpool.find(given_carpool_id))
            .set(&updated_carpool)
            .get_result::<Carpool>(conn)
            .is_ok()
    }

    // pub fn override_table_with_values(
    //     new_vals: &Vec<NewCarpool>,
    //     conn: &MysqlConnection,
    // ) -> QueryResult<()> {
    //     diesel::delete(carpools).execute(conn)?;
    //     diesel::sql_query("ALTER SEQUENCE carpools_id_seq RESTART").execute(conn)?;
    //     diesel::insert_into(carpools)
    //         .values(new_vals)
    //         .execute(conn)?;

    //     Ok(())
    // }
}

#[derive(Debug, Serialize)]
pub struct EventCarpools {
    pub event: Event,
    pub carpools: Vec<EventCarpool>,
}

#[derive(Debug, Serialize)]
pub struct EventCarpool {
    pub driver: Member,
    pub carpool: Carpool,
    pub passengers: Vec<Member>,
}

pub struct NewCarpool {
    pub event: i32,
    pub driver: String,
}
