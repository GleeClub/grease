use crate::db_conn::DbConn;
use chrono::NaiveDateTime;
use crate::models::semester::Semester;
use async_graphql::{Result, SimpleObject, InputObject};

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

    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::load_opt(name, conn).await?.ok_or_else(|| format!("No fee named {}", name))
    }

    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee WHERE name = ?", name)
            .query_optional(conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee ORDER BY NAME")
            .query_all(conn).await
    }

    pub async fn set_amount(name: &str, new_amount: i32, conn: &DbConn) -> Result<()> {
        sqlx::query!("UPDATE fee SET amount = ? WHERE name = ?", new_amount, name).exec(conn).await
    }

    pub async fn charge_dues_for_semester(conn: &DbConn) -> Result<()> {
        let dues = Self::load(Self::DUES, conn).await?;
        let current_semester = Semester::current(conn).await?;

            let members_who_havent_paid = sqlx::query!(
                "SELECT member FROM active_semester WHERE semester = ? AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = ? AND description = ?)",
                 current_semester.name, Self::DUES_NAME, Self::DUES_DESCRIPTION)
                .query_all(conn).await?;

            for email in members_who_havent_paid {
                sqlx::query!(
                    "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES (?, ?, ?, ?, ?)", email, dues.amount, Self::DUES_NAME, Self::DUES_DESCRIPTION,
                     current_semester.name,
                ).query().await?

            }

            Ok(())
    }

    pub async fn charge_late_dues_for_semester(conn: &DbConn) -> Result<()> {
        let late_dues = Self::load(Self::LATE_DUES, conn).await?;
        let current_semester = Semester::current(conn).await?;

            let members_who_havent_paid = sqlx::query!(
                "SELECT member FROM active_semester WHERE semester = ? AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = ? AND description = ?)",
                 current_semester.name, Self::DUES_NAME, Self::DUES_DESCRIPTION)
                .query_all(conn).await?;

            for email in members_who_havent_paid {
                sqlx::query!(
                    "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES (?, ?, ?, ?, ?)", email, late_dues.amount, Self::DUES_NAME, Self::DUES_DESCRIPTION,
                     current_semester.name,
                ).query(conn).await?;
            }

            Ok(())
    }
}

pub struct TransactionType {
    pub name: String,
}

impl TransactionType {
    pub fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type ORDER BY name").query_all(conn).await
    }

    pub async fn with_name(name: &str, conn: &DbConn) -> Result<Self> {
        Self::with_name_opt(name, conn).await?.ok_or_else(|| format!("No transaction type named {}", name))
    }
    
    pub async fn with_name_opt(name: &str, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type WHERE name = ?", name)
            .query_optional(conn).await
    }
}

pub struct ClubTransaction {
    /// The ID of the transaction
    pub id: isize,
    /// The member this transaction was charged to
    pub member: String,
    /// When this transaction was charged
    pub time: NaiveDateTime,
    /// How much this transaction was for
    pub amount: isize,
    /// A description of what the member was charged for specifically
    pub description: String,
    /// Optionally, the name of the semester this tranaction was made during
    pub semester: Option<String>,
    /// The name of the type of transaction
    pub r#type: String,
    /// Whether the member has paid the amount requested in this transaction
    pub resolved: bool
}

impl ClubTransaction {
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| anyhow::anyhow!("No transaction with id {}", id))
    }
    
    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction WHERE id = ?", id)
            .query_optional(conn).await
    }

    pub async fn for_semester(semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction WHERE semester = ? ORDER BY time", semester)
            .query_all(conn).await
    }

    pub async fn for_member_during_semester(member: &str, semester: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction WHERE semester = ? AND member = ? ORDER BY time", member, semester)
            .query_all(conn).await
    }

    pub async fn add_batch(batch: TransactionBatch, conn: &DbConn) -> Result<()> {
        let transaction_type = TransactionType::with_name(batch.r#type, conn).await?;
        
        for member in batch.members {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester) VALUES (?, ?, ?, ?, ?)",
                member, batch.amount, transaction_type.name, batch.description, current_semester.name)
                .query(conn).await
        }

        Ok(())
    }

    pub async fn resolve(id: isize, resolved: bool, conn: &DbConn) -> Result<()> {
        sqlx::query!("UPDATE transaction SET resolved = ? WHERE id = ?", resolved, id)
            .query(conn).await
    }
}

#[derive(InputObject)]
pub struct TransactionBatch {
    pub members: Vec<String>,
    pub r#type: String,
    pub amount: isize,
    pub description: String,
}
