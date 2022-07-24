use async_graphql::{
    InputObject, InputValueError, InputValueResult, Result, Scalar, ScalarType, SimpleObject, Value,
};
use regex::Regex;
use sqlx::PgPool;

#[derive(SimpleObject)]
pub struct Uniform {
    /// The ID of the uniform
    pub id: i64,
    /// The name of the uniform
    pub name: String,
    /// The associated color (In the format #HHH, H being a hex digit)
    pub color: Option<UniformColor>,
    /// The explanation of what to wear when wearing the uniform
    pub description: String,
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
    pub async fn with_id(id: i64, pool: &PgPool) -> Result<Self> {
        Self::with_id_opt(id, pool)
            .await?
            .ok_or_else(|| format!("No uniform with ID {}", id).into())
    }

    pub async fn with_id_opt(id: i64, pool: &PgPool) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniforms WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniforms ORDER BY name"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_default(pool: &PgPool) -> Result<Self> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, color as \"color: _\", description
             FROM uniforms ORDER BY name"
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| "There are currently no uniforms".into())
    }

    pub async fn create(new_uniform: NewUniform, pool: &PgPool) -> Result<i64> {
        sqlx::query!(
            "INSERT INTO uniforms (name, color, description) VALUES ($1, $2, $3)",
            new_uniform.name,
            new_uniform.color.map(|c| c.0),
            new_uniform.description
        )
        .execute(pool)
        .await?;

        sqlx::query_scalar!("SELECT id FROM uniforms ORDER BY id DESC")
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(id: i64, update: NewUniform, pool: &PgPool) -> Result<()> {
        // verify exists
        Uniform::with_id(id, pool).await?;

        sqlx::query!(
            "UPDATE uniforms SET name = $1, color = $2, description = $3 WHERE id = $4",
            update.name,
            update.color as _,
            update.description,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(id: i64, pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM uniforms WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(InputObject)]
pub struct NewUniform {
    pub name: String,
    pub color: Option<UniformColor>,
    pub description: String,
}
