use db::models::member::MemberForSemester;
use db::{Fee, NewTransaction, Semester, Transaction};
use diesel::Connection;
use error::*;

impl Transaction {
    pub fn load_all_for_member<C: Connection>(
        given_member: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Transaction>> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(member.eq(given_member))
            .order_by(time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_of_type_for_semester<C: Connection>(
        given_type: &str,
        given_semester: &str,
        conn: &mut C,
    ) -> GreaseResult<Vec<Transaction>> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(semester.eq(given_semester).and(type_.eq(given_type)))
            .order_by(time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }
}

impl Fee {
    pub fn load<C: Connection>(given_name: &str, conn: &mut C) -> GreaseResult<Fee> {
        use db::schema::fee::dsl::*;

        fee.filter(name.eq(given_name))
            .first(conn)
            .map_err(GreaseError::DbError)
        // format!("No fee with name {}.", name),
    }

    pub fn charge_for_the_semester<C: Connection>(&self, conn: &mut C) -> GreaseResult<()> {
        use db::schema::transaction_type::dsl::*;

        conn.transaction(|| {
            let current_semester = Semester::load_current(conn)?;
            let trans_type = match self.name.as_str() {
                "dues" | "latedues" => "Dues".to_owned(),
                other => transaction_type
                    .filter(name.eq(other))
                    .first(conn)
                    .optional()
                    .map_err(GreaseError::DbError)?
                    .map(|type_| type_.name)
                    .unwrap_or("Other".to_owned()),
            };

            if Transaction::load_all_of_type_for_semester(
                &trans_type,
                &current_semester.name,
                conn,
            )?
            .len()
                > 0
            {
                Err(GreaseError::BadRequest(format!(
                    "Fee of type {} has already been charged for the current semester",
                    &self.name
                )))
            } else {
                let new_transactions = MemberForSemester::load_all(&current_semester.name, conn)?
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
                diesel::insert_into(crate::db::schema::transaction::table)
                    .values(new_transactions)
                    .execute(conn)
                    .map_err(GreaseResult::DbError)
            }
        })
    }

    pub fn load_all<C: Connection>(conn: &mut C) -> GreaseResult<Vec<Fee>> {
        use db::schema::fee::dsl::*;

        fee.order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update_amount<C: Connection>(
        given_name: &str,
        new_amount: i32,
        conn: &mut C,
    ) -> GreaseResult<()> {
        use db::schema::fee::dsl::*;

        diesel::update(fee.filter(name.eq(given_name)))
            .set(amount.eq(new_amount))
            .execute(conn)
            .map_err(GreaseError::DbError)
        // format!("No fee with name {}.", name),
    }
}
