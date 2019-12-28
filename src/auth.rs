//! Authorization handling for the API.
//!
//! The [User](crate::auth::User) struct, in use as an [extract](crate::extract::Extract)able
//! parameter for endpoints, is the primary method for handling authorization
//! for the API.
use db::models::member::MemberForSemester;
use diesel::{MysqlConnection, Queryable};
use error::{GreaseError, GreaseResult};
use serde::{Deserialize, Serialize};

/// The "standard package" for API interaction.
///
/// Simply [extract](crate::extract::Extract) a `User` as an endpoint
/// parameter to assert that a member is logged in to the API. Use the `conn`
/// field to interact with the database and [check_for_permission](../macro.check_for_permission.html)
/// for concise permission checking.
pub struct User {
    /// The user's associated member and potentially their active record
    /// for the current semester.
    pub member: MemberForSemester,
    /// The member's current permissions
    pub permissions: Vec<MemberPermission>,
    /// A connection to the database
    pub conn: MysqlConnection,
}

impl User {
    /// Checks if a member has a permission.
    ///
    /// If `event_type` is None, simply searches the user's permissions. If
    /// it is not None, searches either for a static permission or a permission
    /// with the given event type.
    pub fn has_permission(&self, permission_name: &str, event_type: Option<&str>) -> bool {
        self.permissions.iter().any(|permission| {
            permission.name == permission_name
                && (permission.event_type.is_none()
                    || permission.event_type.as_ref().map(|type_| type_.as_str()) == event_type)
        })
    }

    /// Extract a member from a request.
    ///
    /// Checks for a header named "token" containing the API token
    /// to authenticate a request as from the current user.
    pub fn from_request(request: &cgi::Request) -> GreaseResult<User> {
        let token = request
            .headers()
            .get("token")
            .and_then(|t| t.to_str().ok())
            .ok_or(GreaseError::Unauthorized)?;
        let mut conn = crate::db::connect_to_db()?;
        let member = MemberForSemester::load_from_token(&token, &mut conn)?;
        let permissions = member.member.permissions(&mut conn)?;

        Ok(User {
            member,
            permissions,
            conn,
        })
    }
}

/// The required format for modifying role permissions.
///
/// ## Expected Format:
///
/// |   Field   |  Type  | Required? | Comments |
/// |-----------|--------|:---------:|----------|
/// | name      | string |     ✓     |          |
/// | eventType | string |           |          |
#[derive(Queryable, PartialEq, Serialize, Deserialize)]
pub struct MemberPermission {
    pub name: String,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
}

/// A "one-liner" guard pattern to ensure the user has a given permission.
///
/// This macro checks whether a member is permitted to perform an action, and
/// will return early from an endpoint if they do not. It leverages
/// [User](crate::auth::User)'s [has_permission](crate::auth::User::has_permission)
/// method to check for permissions. This macro is invoked two different ways:
///
/// 1) For static permissions:
///
/// ```rust
/// check_for_permission!(user => "<permission name>");
/// ```
///
/// 2) For event type specific permissions:
///
/// ```rust
/// check_for_permission!(user => "<permission name>", "<event type>");
/// ```
#[macro_export]
macro_rules! check_for_permission {
    ($user:expr => $permission:expr) => {
        if !$user.has_permission($permission, None) {
            return Err(crate::error::GreaseError::Forbidden(Some(
                $permission.to_owned(),
            )));
        }
    };
    ($user:expr => $permission:expr, $event_type:expr) => {
        if !$user.has_permission($permission, Some($event_type)) {
            return Err(crate::error::GreaseError::Forbidden(Some(
                $permission.to_owned(),
            )));
        }
    };
}
