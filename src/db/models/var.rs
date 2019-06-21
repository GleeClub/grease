use db::models::Var;
use db::schema::vars::dsl::*;
use mysql::Conn;
use std::fmt::Display;
use std::str::FromStr;

impl Var {
    pub fn get<T: FromStr>(given_name: &str, conn: &Conn) -> Option<T> {
        vars.filter(name.eq(given_name))
            .first(conn)
            .optional()
            .expect("error getting variable")
            .and_then(|v: Var| v.value.parse().ok())
    }

    pub fn set<T: Display>(given_name: &str, given_value: &T, conn: &Conn) {
        let new_var = Var {
            name: given_name.to_string(),
            value: given_value.to_string(),
        };

        diesel::insert_into(vars)
            .values(&new_var)
            .on_conflict(name)
            .do_update()
            .set(&new_var)
            .execute(conn)
            .ok();
    }

    pub fn unset(given_name: &str, conn: &Conn) {
        diesel::delete(vars.filter(name.eq(given_name)))
            .execute(conn)
            .ok();
    }
}
