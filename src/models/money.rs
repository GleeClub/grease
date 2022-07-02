use async_graphql::{InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::models::semester::Semester;
use crate::models::GqlDateTime;

#[derive(SimpleObject)]
pub struct Fee {
    /// The short name of the fee
    pub name: String,
    /// A longer description of what it is charging members for
    pub description: String,
    /// The amount to charge members
    pub amount: i64,
}

impl Fee {
    pub const DUES: &'static str = "dues";
    pub const LATE_DUES: &'static str = "latedues";

    pub const DUES_NAME: &'static str = "Dues";
    pub const DUES_DESCRIPTION: &'static str = "Semesterly Dues";
    pub const LATE_DUES_DESCRIPTION: &'static str = "Late Dues";

    pub async fn with_name(name: &str, pool: &PgPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No fee named {}", name))
            .map_err(Into::into)
    }

    pub async fn with_name_opt(name: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee WHERE name = $1", name)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee ORDER BY NAME")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn set_amount(name: &str, new_amount: i64, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "UPDATE fee SET amount = $1 WHERE name = $2",
            new_amount,
            name
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn charge_dues_for_semester(pool: &PgPool) -> Result<()> {
        let dues = Self::with_name(Self::DUES, pool).await?;
        let current_semester = Semester::get_current(pool).await?;

        let members_who_havent_paid = sqlx::query_scalar!(
            "SELECT member FROM active_semester WHERE semester = $1 AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = $2 AND description = $3)",
            current_semester.name,
            Self::DUES_NAME,
            Self::DUES_DESCRIPTION
        )
        .fetch_all(pool)
        .await?;

        for email in members_who_havent_paid {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES ($1, $2, $3, $4, $5)",
                email,
                dues.amount,
                Self::DUES_NAME,
                Self::DUES_DESCRIPTION,
                current_semester.name,
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn charge_late_dues_for_semester(pool: &PgPool) -> Result<()> {
        let late_dues = Self::with_name(Self::LATE_DUES, pool).await?;
        let current_semester = Semester::get_current(pool).await?;

        let members_who_havent_paid = sqlx::query_scalar!(
            "SELECT member FROM active_semester WHERE semester = $1 AND member NOT IN \
                 (SELECT member FROM transaction WHERE type = $2 AND description = $3)",
            current_semester.name,
            Self::DUES_NAME,
            Self::DUES_DESCRIPTION
        )
        .fetch_all(pool)
        .await?;

        for email in members_who_havent_paid {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES ($1, $2, $3, $4, $5)",
                email,
                late_dues.amount,
                Self::DUES_NAME,
                Self::LATE_DUES_DESCRIPTION,
                current_semester.name,
            )
            .execute(pool)
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
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn with_name(name: &str, pool: &PgPool) -> Result<Self> {
        Self::with_name_opt(name, pool)
            .await?
            .ok_or_else(|| format!("No transaction type named {}", name))
            .map_err(Into::into)
    }

    pub async fn with_name_opt(name: &str, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM transaction_type WHERE name = $1", name)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }
}

#[derive(SimpleObject)]
pub struct ClubTransaction {
    /// The ID of the transaction
    pub id: i64,
    /// The member this transaction was charged to
    pub member: String,
    /// When this transaction was charged
    pub time: GqlDateTime,
    /// How much this transaction was for
    pub amount: i64,
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
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No transaction with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, member, \"time\" as \"time: _\", amount, description, semester, type, resolved
             FROM transaction WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_semester(semester: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, member, \"time\" as \"time: _\", amount, description, semester, type, resolved
             FROM transaction WHERE semester = $1 ORDER BY time",
            semester
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn for_member(member: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, member, \"time\" as \"time: _\", amount, description, semester, type, resolved
             FROM transaction WHERE member = $1 ORDER BY time",
            member
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn add_batch(batch: TransactionBatch, pool: &PgPool) -> Result<()> {
        let current_semester = Semester::get_current(pool).await?;
        let transaction_type = TransactionType::with_name(&batch.r#type, pool).await?;

        for member in batch.members {
            sqlx::query!(
                "INSERT INTO transaction (member, amount, type, description, semester) VALUES ($1, $2, $3, $4, $5)",
                member, batch.amount, transaction_type.name, batch.description, current_semester.name)
                .execute(pool).await?;
        }

        Ok(())
    }

    pub async fn resolve(id: i64, resolved: bool, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "UPDATE transaction SET resolved = $1 WHERE id = $2",
            resolved,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct TransactionBatch {
    pub members: Vec<String>,
    pub r#type: String,
    pub amount: i64,
    pub description: String,
}
