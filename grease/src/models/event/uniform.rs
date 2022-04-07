use async_graphql::{SimpleObject, ComplexObject, InputValueError, InputValueResult, ScalarType, Scalar, InputObject};
use serde_json::Value;
use crate::db_conn::DbConn;
use regex::Regex;

#[derive(SimpleObject)]
pub struct Uniform {
    /// The ID of the uniform
    pub id: isize,
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
        if let Value::String(value) = &value {
            let regex = Regex::new(r"^#(\w{3}|\w{6})$").unwrap();
            if regex.is_match(value) {
                Ok(UniformColor(value))
            } else {
                Err(InputValueError::custom("Uniform colors must look like #RGB or #RRGGBB"))
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
    pub async fn with_id(id: isize, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn).await?.ok_or_else(|| format!("No uniform with id {}", id))
    }

    pub async fn with_id_opt(id: isize, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM uniform WHERE id = ?", id).query_optional(conn).await
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM uniform ORDER BY name").query_all(conn).await
    }

    pub async fn get_default(conn: &DbConn) -> Result<Self> {
        let uniform = sqlx::query_as!(Self, "SELECT * FROM uniform ORDER BY NAME").query_optional(conn).await?;
        uniform.ok_or_else(|| "There are currently no uniforms")
    }

    pub async fn create(new_uniform: NewUniform, conn: &DbConn) -> Result<isize> {
        sqlx::query!(
            "INSERT INTO uniform (name, color, description) VALUES (?, ?, ?)",
            new_uniform.name, new_uniform.color, new_uniform.description).query(conn).await?;

        sqlx::query!("SELECT id FROM uniform ORDER BY id DESC").query(conn).await
    }

    pub async fn update(id: isize, updated_uniform: UniformUpdate,conn: &DbConn) -> Result<()> {
        // TODO: verify exists?
        // TODO: mutation?
        sqlx::query!(
            "UPDATE uniform SET name = ?, color = ?, description = ? WHERE id = ?",
            update.name, update.color, update.description, id).query(conn).await
    }

    pub async fn delete(id: isize, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM uniform WHERE id = ?", id).query(conn).await
    }
}

#[derive(InputObject)]
pub struct NewUniform {
    pub name: String,
    pub color: Option<UniformColor>,
    pub description: Option<String>,
}
