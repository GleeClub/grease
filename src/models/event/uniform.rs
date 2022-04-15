use async_graphql::{
    InputObject, InputValueError, InputValueResult, Result, Scalar, ScalarType, SimpleObject, Value,
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
#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct UniformColor(String);

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
    pub async fn with_id(id: i32, conn: &DbConn) -> Result<Self> {
        Self::with_id_opt(id, conn)
            .await?
            .ok_or_else(|| format!("No uniform with id {}", id).into())
    }

    pub async fn with_id_opt(id: i32, conn: &DbConn) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniform WHERE id = ?",
            id
        )
        .fetch_optional(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniform ORDER BY name"
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }

    pub async fn get_default(conn: &DbConn) -> Result<Self> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniform ORDER BY name"
        )
        .fetch_optional(&mut *conn.get().await)
        .await?
        .ok_or_else(|| "There are currently no uniforms".into())
    }

    pub async fn create(new_uniform: NewUniform, conn: &DbConn) -> Result<i32> {
        sqlx::query!(
            "INSERT INTO uniform (name, color, description) VALUES (?, ?, ?)",
            new_uniform.name,
            new_uniform.color,
            new_uniform.description
        )
        .execute(&mut *conn.get().await)
        .await?;

        sqlx::query_scalar!("SELECT id FROM uniform ORDER BY id DESC")
            .fetch_one(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i32, update: NewUniform, conn: &DbConn) -> Result<()> {
        // TODO: verify exists?
        // TODO: mutation?
        sqlx::query!(
            "UPDATE uniform SET name = ?, color = ?, description = ? WHERE id = ?",
            update.name,
            update.color,
            update.description,
            id
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn delete(id: i32, conn: &DbConn) -> Result<()> {
        sqlx::query!("DELETE FROM uniform WHERE id = ?", id)
            .execute(&mut *conn.get().await)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewUniform {
    pub name: String,
    pub color: Option<UniformColor>,
    pub description: Option<String>,
}
