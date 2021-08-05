use anyhow::{Context, Result};

pub struct Semester {
    /// The name of the semester
    pub name: String,
    /// When the semester starts
    pub start_date: NaiveDateTime,
    /// When the semester ends
    pub end_date: NaiveDateTime,
    /// How many volunteer gigs are required for the semester
    pub gig_requirement: i32, // default = 5
    /// Whether this is the new semester
    pub current: bool, // default = false
}

impl Semester {
    pub fn current(conn: &mut MysqlConnection) -> Result<Self> {
        sqlx::query_as!(Semester, "SELECT * FROM semester WHERE current = true")
            .query_optional(conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No current semester set"))
    }

    pub fn with_name_opt(name: &str, conn: &mut MysqlConnection) -> Result<Self> {
        sqlx::query_as!(Semester, "SELECT * FROM semester WHERE name = ?", name)
            .query_optional(conn)
            .await
            .into()
    }

    pub fn with_name(name: &str, conn: &mut MysqlConnection) -> Result<Self> {
        Self::with_name_opt(name, conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No semester named {}", name))
    }

    pub fn all(conn: &mut MysqlConnection) -> Result<Vec<Self>> {
        sqlx::query_as!(Semester, "SELECT * FROM semester ORDER BY start_date")
            .query_all(conn)
            .await
            .into()
    }

    pub fn create(new: NewSemester, conn: &mut MysqlConnection) -> Result<()> {
        if Self::with_name_opt(&new.name, conn).await?.is_some() {
            anyhow::bail!("A semester already exists named {}", new.name);
        }

        sqlx::query!(
            "INSERT INTO semester (name, start_date, end_date, gig_requirement)
             VALUES (?, ?, ?, ?)",
            new.name,
            new.start_date,
            new.end_date,
            new.gig_requirement
        )
        .query(conn)
        .await
        .into()
    }

    pub fn update(name: &str, update: SemesterUpdate, conn: &mut MysqlConnection) -> Result<()> {
        // check that semester exists
        Self::with_name(name, conn).await?;

        if name != &update.name && Self::with_name_opt(&update.name, conn).await?.is_some() {
            anyhow::bail!("Another semester is already named {}", update.name);
        }

        sqlx::query!(
            "UPDATE semester SET
             name = ?, start_date = ?, end_date = ?, gig_requirement = ?
             WHERE name = ?",
            update.name,
            update.start_date,
            update.end_date,
            update.gig_requirement,
            update.name
        )
        .query(conn)
        .await
        .into()
    }

    pub fn set_current(name: &str, conn: &mut MysqlConnection) -> Result<()> {
        conn.begin(|tx| {
            if sqlx::query!("SELECT 1 FROM semester WHERE name = ?", name)
                .query_optional(tx)
                .await?
                .is_none()
            {
                anyhow::bail!("No semester named {}", name);
            }

            sqlx::query!("UPDATE semester SET current = false")
                .query(tx)
                .await?;
            sqlx::query!("UPDATE semester SET current = true WHERE name = ?", true)
                .query(tx)
                .await?;

            Ok(())
        })
        .await
    }
}
