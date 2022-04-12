use async_graphql::{
    Value, InputObject, InputValueError, InputValueResult, Scalar, ScalarType, SimpleObject,
};
use regex::Regex;
use crate::db::DbConn;

#[derive(SimpleObject)]
pub struct Uniform {
    /// The ID of the uniform
    pub id: i32,
    /// The name of the uniform
    pub name: String,
    /// The associated color (In the format #HHH, H being a hex digit)
    pub color: Option<UniformColor>,
    /// The explanation of what to wear when wearing the uniform
    pub description: Option<String>,
}

/// A color for a uniform when rendered on the site
struct UniformColor(String);

#[Scalar]
impl ScalarType for UniformColor {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(color) = value {
            let regex = Regex::new(r"^#(\w{3}|\w{6})$").unwrap();
            if regex.is_match(&color) {
                Ok(UniformColor(color))
            } else {
                Err(InputValueError::custom(
                    "Uniform colors must look like #RGB or #RRGGBB",
                ))
            }
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_owned())
    }
}

impl Uniform {
    pub async fn with_id(id: i32, mut conn: DbConn<'_>) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| format!("No uniform with id {}", id))
    }

    pub async fn with_id_opt(id: i32, mut conn: DbConn<'_>) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM uniform WHERE id = ?", id)
            .fetch_optional(conn)
            .await
    }

    pub async fn all(mut conn: DbConn<'_>) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM uniform ORDER BY name")
            .fetch_all(conn)
            .await
    }

    pub async fn get_default(mut conn: DbConn<'_>) -> Result<Self> {
        let uniform = sqlx::query_as!(Self, "SELECT * FROM uniform ORDER BY NAME")
            .fetch_optional(conn)
            .await?;
        uniform.ok_or_else(|| "There are currently no uniforms")
    }

    pub async fn create(new_uniform: NewUniform, mut conn: DbConn<'_>) -> Result<i32> {
        sqlx::query!(
            "INSERT INTO uniform (name, color, description) VALUES (?, ?, ?)",
            new_uniform.name,
            new_uniform.color,
            new_uniform.description
        )
        .execute(conn)
        .await?;

        sqlx::query!("SELECT id FROM uniform ORDER BY id DESC")
            .execute(conn)
            .await
    }

    pub async fn update(id: i32, update: NewUniform, mut conn: DbConn<'_>) -> Result<()> {
        // TODO: verify exists?
        // TODO: mutation?
        sqlx::query!(
            "UPDATE uniform SET name = ?, color = ?, description = ? WHERE id = ?",
            update.name,
            update.color,
            update.description,
            id
        )
        .execute(conn)
        .await
    }

    pub async fn delete(id: i32, mut conn: DbConn<'_>) -> Result<()> {
        sqlx::query!("DELETE FROM uniform WHERE id = ?", id)
            .execute(conn)
            .await
    }
}

#[derive(InputObject)]
pub struct NewUniform {
    pub name: String,
    pub color: Option<UniformColor>,
    pub description: Option<String>,
}
