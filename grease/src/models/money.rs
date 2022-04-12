use async_graphql::{InputObject, Result, SimpleObject};
use time::OffsetDateTime;

use crate::db::DbConn;
use crate::models::semester::Semester;

#[derive(SimpleObject)]
pub struct Fee {
    /// The short name of the fee
    pub name: String,
    /// A longer description of what it is charging members for
    pub description: String,
    /// The amount to charge members
    pub amount: i32,
}

impl Fee {
    pub const DUES: &'static str = "dues";
    pub const LATE_DUES: &'static str = "latedues";

    pub const DUES_NAME: &'static str = "Dues";
    pub const DUES_DESCRIPTION: &'static str = "Semesterly Dues";
    pub const LATE_DUES_DESCRIPTION: &'static str = "Late Dues";

    pub async fn with_name(name: &str, conn: DbConn<'_>) -> Result<Self> {
        Self::load_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No fee named {}", name))
    }

    pub async fn with_name_opt(name: &str, conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee WHERE name = ?", name)
            .fetch_optional(conn)
            .await
    }

    pub async fn all(conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee ORDER BY NAME")
            .fetch_all(conn)
            .await
    }

    pub async fn set_amount(name: &str, new_amount: i32, conn: DbConn<'_>) -> Result<()> {
        sqlx::query!("UPDATE fee SET amount = ? WHERE name = ?", new_amount, name)
            .exec(conn)
            .await
    }

    pub async fn charge_dues_for_semester(conn: DbConn<'_>) -> Result<()> {
        let dues = Self::load(Self::DUES, conn).await?;
        let current_semester = Semester::current(conn).await?;

        let members_who_havent_paid = sqlx::query!(
            "SELECT member FROM active_semester WHERE semester = ? AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = ? AND description = ?)",
            current_semester.name,
            Self::DUES_NAME,
            Self::DUES_DESCRIPTION
        )
        .fetch_all(conn)
        .await?;

        for email in members_who_havent_paid {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES (?, ?, ?, ?, ?)",
                email,
                dues.amount,
                Self::DUES_NAME,
                Self::DUES_DESCRIPTION,
                current_semester.name,
            )
            .execute()
            .await?
        }

        Ok(())
    }

    pub async fn charge_late_dues_for_semester(conn: DbConn<'_>) -> Result<()> {
        let late_dues = Self::load(Self::LATE_DUES, conn).await?;
        let current_semester = Semester::current(conn).await?;

        let members_who_havent_paid = sqlx::query!(
            "SELECT member FROM active_semester WHERE semester = ? AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = ? AND description = ?)",
            current_semester.name,
            Self::DUES_NAME,
            Self::DUES_DESCRIPTION
        )
        .fetch_all(conn)
        .await?;

        for email in members_who_havent_paid {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES (?, ?, ?, ?, ?)",
                email,
                late_dues.amount,
                Self::DUES_NAME,
                Self::DUES_DESCRIPTION,
                current_semester.name,
            )
            .execute(conn)
            .await?;
        }

        Ok(())
    }
}

#[derive(SimpleObject)]
pub struct TransactionType {
    pub name: String,
}

impl TransactionType {
    pub async fn all(conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type ORDER BY name")
            .fetch_all(conn)
            .await
    }

    pub async fn with_name(name: &str, conn: DbConn<'_>) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| format!("No transaction type named {}", name))
            .into()
    }

    pub async fn with_name_opt(name: &str, conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type WHERE name = ?", name)
            .fetch_optional(conn)
            .await
    }
}

pub struct ClubTransaction {
    /// The ID of the transaction
    pub id: i32,
    /// The member this transaction was charged to
    pub member: String,
    /// When this transaction was charged
    pub time: OffsetDateTime,
    /// How much this transaction was for
    pub amount: i32,
    /// A description of what the member was charged for specifically
    pub description: String,
    /// Optionally, the name of the semester this tranaction was made during
    pub semester: Option<String>,
    /// The name of the type of transaction
    pub r#type: String,
    /// Whether the member has paid the amount requested in this transaction
    pub resolved: bool,
}

impl ClubTransaction {
    pub async fn with_id(id: i32, conn: DbConn<'_>) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No transaction with id {}", id))
    }

    pub async fn with_id_opt(id: i32, conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn for_semester(semester: &str, conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM transaction WHERE semester = ? ORDER BY time",
            semester
        )
        .fetch_all(conn)
        .await
    }

    pub async fn for_member_during_semester(
        member: &str,
        semester: &str,
        conn: DbConn<'_>,
    ) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM transaction WHERE semester = ? AND member = ? ORDER BY time",
            member,
            semester
        )
        .fetch_all(conn)
        .await
    }

    pub async fn add_batch(batch: TransactionBatch, conn: DbConn<'_>) -> Result<()> {
        let current_semester = Semester::current(conn).await?;
        let transaction_type = TransactionType::with_name(batch.r#type, conn).await?;

        for member in batch.members {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester) VALUES (?, ?, ?, ?, ?)",
                member, batch.amount, transaction_type.name, batch.description, current_semester.name)
                .execute(conn).await
        }

        Ok(())
    }

    pub async fn resolve(id: i32, resolved: bool, conn: DbConn<'_>) -> Result<()> {
        sqlx::query!(
            "UPDATE transaction SET resolved = ? WHERE id = ?",
            resolved,
            id
        )
        .execute(conn)
        .await
    }
}

#[derive(InputObject)]
pub struct TransactionBatch {
    pub members: Vec<String>,
    pub r#type: String,
    pub amount: i32,
    pub description: String,
}
