use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};
use sqlx::PgPool;

use crate::models::member::Member;

/// Roles that can be held by members to grant permissions
#[derive(SimpleObject)]
pub struct Role {
    /// The name of the role
    pub name: String,
    /// Used for ordering the positions (e.g. President beforee Ombudsman)
    pub rank: i64,
    /// The maximum number of the position allowed to be held at once.
    /// If it is 0 or less, no maximum is enforced
    pub max_quantity: i64,
}

impl Role {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM roles ORDER BY rank")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn for_member(email: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM roles WHERE name in 
             (SELECT role FROM member_roles WHERE member = $1) 
             ORDER BY rank",
            email
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

/// A member that holds a role
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct MemberRole {
    /// The name of the role being held
    pub role: String,

    #[graphql(skip)]
    pub member: String,
}

#[ComplexObject]
impl MemberRole {
    /// The member holding the role
    pub async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool: &PgPool = ctx.data_unchecked();
        Member::with_email(&self.member, pool).await
    }
}

impl MemberRole {
    pub async fn current_officers(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM member_roles ORDER BY role, member")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn member_has_role(member: &str, role: &str, pool: &PgPool) -> Result<bool> {
        let member_role = sqlx::query!(
            "SELECT * FROM member_roles WHERE member = $1 AND role = $2",
            member,
            role
        )
        .fetch_optional(pool)
        .await?;

        Ok(member_role.is_some())
    }

    pub async fn add(member: &str, role: &str, pool: &PgPool) -> Result<()> {
        if Self::member_has_role(member, role, pool).await? {
            return Err("Member already has that role".into());
        }

        sqlx::query!(
            "INSERT INTO member_roles (member, role) VALUES ($1, $2)",
            member,
            role
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(member: &str, role: &str, pool: &PgPool) -> Result<()> {
        if !Self::member_has_role(member, role, pool).await? {
            return Err("Member does not have that role".into());
        }

        sqlx::query!(
            "DELETE FROM member_roles WHERE member = $1 AND role = $2",
            member,
            role
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Whether a permission applies to a specific event type or not
#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(type_name = "permission_type", rename_all = "snake_case")]
pub enum PermissionType {
    /// The permission applies globally
    Static,
    /// The permission applies for a specific type of event
    Event,
}

/// A permission that grants abilities to a member
#[derive(SimpleObject)]
pub struct Permission {
    /// The name of the permission
    pub name: String,
    /// A description of what the permission entails
    pub description: Option<String>,
    /// Whether the permission applies to a type of event or generally
    pub r#type: PermissionType,
}

impl Permission {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, description, type as \"type: _\"
             FROM permissions ORDER BY name"
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}

/// A new assignment of a permission to a role
#[derive(InputObject)]
pub struct NewRolePermission {
    /// The name of the role this junction refers to
    pub role: String,
    /// The name of the permission the role is awarded
    pub permission: String,
    /// Optionally, the type of the event the permission applies to
    pub event_type: Option<String>,
}

/// A permissin that is inherited by a role
#[derive(SimpleObject)]
pub struct RolePermission {
    /// The ID of the role permission
    pub id: i64,
    /// The name of the role this junction refers to
    pub role: String,
    /// The name of the permission the role is awarded
    pub permission: String,
    /// Optionally, the type of the event the permission applies to
    pub event_type: Option<String>,
}

impl RolePermission {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM role_permissions")
            .fetch_all(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn add(role_permission: NewRolePermission, pool: &PgPool) -> Result<()> {
        sqlx::query_as!(
            Self,
            "INSERT INTO role_permissions (role, permission, event_type)
             VALUES ($1, $2, $3)
             ON CONFLICT(role, permission, event_type) DO NOTHING",
            role_permission.role,
            role_permission.permission,
            role_permission.event_type
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove(role_permission: NewRolePermission, pool: &PgPool) -> Result<()> {
        sqlx::query_as!(
            Self,
            "DELETE FROM role_permissions WHERE role = $1 AND permission = $2 AND event_type = $3",
            role_permission.role,
            role_permission.permission,
            role_permission.event_type
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// A permission that a member holds
#[derive(SimpleObject)]
pub struct MemberPermission {
    /// The name of the permission
    pub name: String,
    /// Optionally, the type of event the permission applies to
    pub event_type: Option<String>,
}

impl MemberPermission {
    pub async fn for_member(member: &str, pool: &PgPool) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT permission as name, event_type FROM role_permissions
             INNER JOIN member_roles ON role_permissions.role = member_roles.role
             WHERE member_roles.member = $1",
            member
        )
        .fetch_all(pool)
        .await
        .map_err(Into::into)
    }
}
