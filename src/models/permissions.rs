use async_graphql::{ComplexObject, Context, Enum, InputObject, Result, SimpleObject};

use crate::db::DbConn;
use crate::models::member::Member;

/// Roles that can be held by members to grant permissions
#[derive(SimpleObject)]
pub struct Role {
    /// The name of the role
    pub name: String,
    /// Used for ordering the positions (e.g. President beforee Ombudsman)
    pub rank: i32,
    /// The maximum number of the position allowed to be held at once.
    /// If it is 0 or less, no maximum is enforced
    pub max_quantity: i32,
}

impl Role {
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM role ORDER BY rank")
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn for_member(email: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
                "SELECT * FROM role WHERE name IN (SELECT rank FROM member_role WHERE member = ?) ORDER BY rank", email)
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }
}

#[derive(SimpleObject)]
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
        let conn = DbConn::from_ctx(ctx);
        Member::with_email(&self.member, conn).await
    }
}

impl MemberRole {
    pub async fn current_officers(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM member_role ORDER BY role, member")
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn member_has_role(member: &str, role: &str, conn: &DbConn) -> Result<bool> {
        let member_role = sqlx::query!(
            "SELECT * FROM member_role WHERE member = ? AND role = ?",
            member,
            role
        )
        .fetch_optional(&mut *conn.get().await)
        .await?;

        Ok(member_role.is_some())
    }

    pub async fn add(member: &str, role: &str, conn: &DbConn) -> Result<()> {
        if Self::member_has_role(member, role, conn).await? {
            return Err("Member already has that role".into());
        }

        sqlx::query!(
            "INSERT INTO member_role (member, role) VALUES (?, ?)",
            member,
            role
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn remove(member: &str, role: &str, conn: &DbConn) -> Result<()> {
        if !Self::member_has_role(member, role, conn).await? {
            return Err("Member does not have that role".into());
        }

        sqlx::query!(
            "DELETE FROM member_role WHERE member = ? AND role = ?",
            member,
            role
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum PermissionType {
    Static,
    Event,
}

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
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT name, description, `type` as \"type: _\"
             FROM permission ORDER BY name"
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }
}

#[derive(InputObject)]
pub struct NewRolePermission {
    /// The name of the role this junction refers to
    pub role: String,
    /// The name of the permission the role is awarded
    pub permission: String,
    /// Optionally, the type of the event the permission applies to
    pub event_type: Option<String>,
}

#[derive(SimpleObject)]
pub struct RolePermission {
    /// The ID of the role permission
    pub id: i32,
    /// The name of the role this junction refers to
    pub role: String,
    /// The name of the permission the role is awarded
    pub permission: String,
    /// Optionally, the type of the event the permission applies to
    pub event_type: Option<String>,
}

impl RolePermission {
    pub async fn all(conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM role_permission")
            .fetch_all(&mut *conn.get().await)
            .await
            .map_err(Into::into)
    }

    pub async fn add(role_permission: NewRolePermission, conn: &DbConn) -> Result<()> {
        sqlx::query_as!(
            Self,
            "INSERT IGNORE INTO role_permission (role, permission, event_type) VALUES (?, ?, ?)",
            role_permission.role,
            role_permission.permission,
            role_permission.event_type
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }

    pub async fn remove(role_permission: NewRolePermission, conn: &DbConn) -> Result<()> {
        sqlx::query_as!(
            Self,
            "DELETE FROM role_permission WHERE role = ? AND permission = ? AND event_type = ?",
            role_permission.role,
            role_permission.permission,
            role_permission.event_type
        )
        .execute(&mut *conn.get().await)
        .await?;

        Ok(())
    }
}

#[derive(SimpleObject)]
pub struct MemberPermission {
    /// The name of the permission
    pub name: String,
    /// Optionally, the type of event the permission applies to
    pub event_type: Option<String>,
}

impl MemberPermission {
    pub async fn for_member(member: &str, conn: &DbConn) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT permission as name, event_type FROM role_permission
             INNER JOIN member_role ON role_permission.role = member_role.role
             WHERE member_role.member = ?",
            member
        )
        .fetch_all(&mut *conn.get().await)
        .await
        .map_err(Into::into)
    }
}
