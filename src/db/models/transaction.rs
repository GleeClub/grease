use db::models::member::MemberForSemester;
use db::*;
use error::*;
use pinto::query_builder::*;

impl Transaction {
    pub fn load_all_for_member<C: Connection>(member: &str, conn: &mut C) -> GreaseResult<Vec<Transaction>> {
        conn.load(Self::filter(&format!("member = '{}'", member)).order_by("time", Order::Asc))
    }

    pub fn load_all_of_type_for_semester<C: Connection>(
        type_: &str,
        semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Transaction>> {
        conn.load(Self::filter(&format!("semester = '{}' AND `type` = '{}'", semester, type_)).order_by("time", Order::Asc))
    }
}

impl Fee {
    pub fn load<C: Connection>(name: &str, conn: &mut C) -> GreaseResult<Fee> {
        conn.first(&Self::filter(&format!("name = '{}'", name)), format!("No fee with name {}.", name))
    }

    pub fn charge_for_the_semester(&self, conn: &mut DbConn) -> GreaseResult<()> {
        conn.transaction(|db_transaction| {
            let current_semester = Semester::load_current(db_transaction)?;
            let transaction_type = match self.name.as_str() {
                "dues" | "latedues" => "Dues".to_owned(),
                other => db_transaction
                    .first_opt::<TransactionType>(&TransactionType::filter(&format!("name = '{}'", other)))?
                    .map(|type_| type_.name)
                    .unwrap_or("Other".to_owned()),
            };

            if Transaction::load_all_of_type_for_semester(
                &transaction_type,
                &current_semester.name,
                db_transaction,
            )?
            .len()
                == 0
            {
                let new_transactions =
                    MemberForSemester::load_all(&current_semester.name, db_transaction)?
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
                    new_transaction.insert(db_transaction)?;
                }

                Ok(())
            } else {
                Err(GreaseError::BadRequest(format!(
                    "Fee of type {} has already been charged for the current semester",
                    &self.name
                )))
            }
        })
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Fee>> {
        conn.load(&Fee::select_all_in_order("name", Order::Asc))
    }

    pub fn update_amount<C: Connection>(name: &str, new_amount: i32, conn: &mut C) -> GreaseResult<()> {
        conn.update(
            Update::new(Self::table_name())
                .filter(&format!("name = '{}'", name))
                .set("amount", &new_amount.to_string()),
            format!("No fee with name {}.", name)
        )
    }
}
