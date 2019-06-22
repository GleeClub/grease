use chrono::Local;
use db::models::member::MemberForSemester;
use db::models::*;
use db::traits::*;
use error::*;
use mysql::{prelude::GenericConnection, Conn};
use pinto::query_builder::{self, Join, Order};
use serde::Serialize;
use serde_json::{json, Value};

impl Transaction {
    pub fn load_all_for_member(member: &str, conn: &mut Conn) -> GreaseResult<Vec<Transaction>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!("member = '{}'", member))
            .order_by("time", Order::Asc)
            .build();

        crate::db::load(&query, conn)
    }

    pub fn load_all_of_type_for_semester<G: GenericConnection>(
        type_: &str,
        semester: &str,
        conn: &mut G,
    ) -> GreaseResult<Vec<Transaction>> {
        let query = query_builder::select(Self::table_name())
            .fields(Self::field_names())
            .filter(&format!(
                "semester = '{}' AND `type` = '{}'",
                semester, type_
            ))
            .build();

        crate::db::load(&query, conn)
    }
}

impl Fee {
    pub fn load(name: &str, conn: &mut Conn) -> GreaseResult<Fee> {
        Fee::first(
            &format!("name = '{}'", name),
            conn,
            format!("no fee with name {}", name),
        )
    }

    pub fn charge_for_the_semester(&self, conn: &mut Conn) -> GreaseResult<()> {
        let mut db_transaction = conn
            .start_transaction(false, None, None)
            .map_err(GreaseError::DbError)?;
        let current_semester = Semester::load_current(&mut db_transaction)?;
        let transaction_type = match self.name.as_str() {
            "dues" | "latedues" => "Dues".to_owned(),
            other => {
                TransactionType::first_opt(&format!("name = '{}'", other), &mut db_transaction)?
                    .map(|type_| type_.name)
                    .unwrap_or("Other".to_owned())
            }
        };

        if Transaction::load_all_of_type_for_semester(
            &transaction_type,
            &current_semester.name,
            &mut db_transaction,
        )?
        .len()
            == 0
        {
            let new_transactions =
                MemberForSemester::load_all(&current_semester.name, &mut db_transaction)?
                    .into_iter()
                    .map(|member_for_semester| NewTransaction {
                        member: member_for_semester.member.email,
                        amount: self.amount,
                        description: self.description.clone(),
                        semester: Some(current_semester.name.clone()),
                        type_: transaction_type.clone(),
                        resolved: true,
                    })
                    .collect::<Vec<_>>();
            for new_transaction in new_transactions {
                new_transaction.insert(&mut db_transaction)?;
            }
            db_transaction.commit().map_err(GreaseError::DbError)?;

            Ok(())
        } else {
            Err(GreaseError::BadRequest(format!(
                "Fee if type '{}' has already been charged for the current semester",
                &self.name
            )))
        }
    }

    pub fn load_all(conn: &mut Conn) -> GreaseResult<Vec<Fee>> {
        Fee::query_all_in_order(vec![("name", Order::Asc)], conn)
    }

    pub fn update_amount(name: &str, new_amount: i32, conn: &mut Conn) -> GreaseResult<()> {
        let query = query_builder::update(Self::table_name())
            .filter(&format!("name = '{}'", name))
            .set("amount", &new_amount.to_string())
            .build();
        conn.query(query).map_err(GreaseError::DbError)?;

        Ok(())
    }
}
