use db::{Fee, NewTransaction, Semester, Transaction, TransactionBatch, TransactionType};
use diesel::prelude::*;
use error::*;

impl Transaction {
    pub const DUES_NAME: &'static str = "Dues";
    pub const DUES_DESCRIPTION: &'static str = "Semesterly Dues";
    pub const LATE_DUES_DESCRIPTION: &'static str = "Late Dues";

    pub fn load(given_id: i32, conn: &MysqlConnection) -> GreaseResult<Transaction> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(id.eq(given_id))
            .first(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "No transaction exists wtih id {}.",
                given_id
            )))
    }

    pub fn load_all_for_member(
        given_member: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Transaction>> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(member.eq(given_member))
            .order_by(time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_for_semester(
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Transaction>> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(semester.eq(given_semester))
            .order_by(time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn load_all_of_type_for_semester(
        given_type: &str,
        given_semester: &str,
        conn: &MysqlConnection,
    ) -> GreaseResult<Vec<Transaction>> {
        use db::schema::transaction::dsl::*;

        transaction
            .filter(semester.eq(given_semester).and(type_.eq(given_type)))
            .order_by(time.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn charge_for_members(batch: TransactionBatch, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::{transaction, transaction_type};

        let current_semester = Semester::load_current(conn)?;
        let type_ = transaction_type::table
            .filter(transaction_type::name.eq(&batch.type_))
            .first::<TransactionType>(conn)
            .optional()?;
        if type_.is_none() {
            return Err(GreaseError::BadRequest(format!(
                "No transaction type called {}.",
                &batch.type_
            )));
        }

        let new_transactions = batch
            .members
            .iter()
            .map(|given_member| NewTransaction {
                member: given_member.clone(),
                amount: batch.amount.clone(),
                type_: batch.type_.clone(),
                description: batch.description.clone(),
                semester: Some(current_semester.name.clone()),
                resolved: false,
            })
            .collect::<Vec<NewTransaction>>();
        diesel::insert_into(transaction::table)
            .values(&new_transactions)
            .execute(conn)?;

        Ok(())
    }

    pub fn resolve(given_id: i32, is_resolved: bool, conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::transaction::dsl::*;

        let _trans = Self::load(given_id, conn)?;
        diesel::update(transaction.filter(id.eq(given_id)))
            .set(resolved.eq(is_resolved))
            .execute(conn)?;

        Ok(())
    }
}

impl Fee {
    pub const DUES: &'static str = "dues";
    pub const LATE_DUES: &'static str = "latedues";

    pub fn load(given_name: &str, conn: &MysqlConnection) -> GreaseResult<Fee> {
        use db::schema::fee::dsl::*;

        fee.filter(name.eq(given_name))
            .first(conn)
            .optional()?
            .ok_or(GreaseError::BadRequest(format!(
                "No fee with name {}.",
                given_name
            )))
    }

    pub fn load_all(conn: &MysqlConnection) -> GreaseResult<Vec<Fee>> {
        use db::schema::fee::dsl::*;

        fee.order_by(name.asc())
            .load(conn)
            .map_err(GreaseError::DbError)
    }

    pub fn update_amount(
        given_name: &str,
        new_amount: i32,
        conn: &MysqlConnection,
    ) -> GreaseResult<()> {
        use db::schema::fee::dsl::*;

        diesel::update(fee.filter(name.eq(given_name)))
            .set(amount.eq(new_amount))
            .execute(conn)
            .map_err(GreaseError::DbError)?;

        Ok(())
        // format!("No fee with name {}.", name),
    }

    pub fn charge_dues_for_semester(conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::{active_semester, member, transaction};

        let current_semester = Semester::load_current(conn)?;
        let dues = Self::load(Fee::DUES, conn)?;

        let members_who_have_not_paid = member::table
            .select(member::email)
            .filter(
                member::email
                    .eq_any(
                        active_semester::table
                            .filter(active_semester::semester.eq(&current_semester.name))
                            .select(active_semester::member),
                    )
                    .and(
                        member::email.ne_all(
                            transaction::table
                                .filter(transaction::type_.eq(Transaction::DUES_NAME).and(
                                    transaction::description.eq(Transaction::DUES_DESCRIPTION),
                                ))
                                .select(transaction::member),
                        ),
                    ),
            )
            .load(conn)?;

        let new_transactions = members_who_have_not_paid
            .into_iter()
            .map(|given_member| NewTransaction {
                member: given_member,
                amount: dues.amount,
                type_: Transaction::DUES_NAME.to_owned(),
                description: Transaction::DUES_DESCRIPTION.to_owned(),
                semester: Some(current_semester.name.clone()),
                resolved: false,
            })
            .collect::<Vec<NewTransaction>>();

        diesel::insert_into(transaction::table)
            .values(&new_transactions)
            .execute(conn)?;

        Ok(())
    }

    pub fn charge_late_dues_for_semester(conn: &MysqlConnection) -> GreaseResult<()> {
        use db::schema::{active_semester, member, transaction};

        let current_semester = Semester::load_current(conn)?;
        let late_dues = Self::load(Fee::LATE_DUES, conn)?;

        let members_who_have_not_paid_dues = member::table
            .select(member::email)
            .filter(
                member::email
                    .eq_any(
                        active_semester::table
                            .filter(active_semester::semester.eq(&current_semester.name))
                            .select(active_semester::member),
                    )
                    .and(
                        member::email.eq_any(
                            transaction::table
                                .filter(
                                    transaction::type_
                                        .eq(Transaction::DUES_NAME)
                                        .and(
                                            transaction::description
                                                .eq(Transaction::DUES_DESCRIPTION),
                                        )
                                        .and(transaction::resolved.eq(false)),
                                )
                                .select(transaction::member),
                        ),
                    ),
            )
            .load(conn)?;

        let new_transactions = members_who_have_not_paid_dues
            .into_iter()
            .map(|given_member| NewTransaction {
                member: given_member,
                amount: late_dues.amount,
                type_: Transaction::DUES_NAME.to_owned(),
                description: Transaction::LATE_DUES_DESCRIPTION.to_owned(),
                semester: Some(current_semester.name.clone()),
                resolved: false,
            })
            .collect::<Vec<NewTransaction>>();

        diesel::insert_into(transaction::table)
            .values(&new_transactions)
            .execute(conn)?;

        Ok(())
    }
}
