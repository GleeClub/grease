//! Authorization handling for the API.
//!
//! The [User](crate::auth::User) struct, in use as an [extract](crate::extract::Extract)able
//! parameter for endpoints, is the primary method for handling authorization
//! for the API.
use db::{member::MemberForSemester, DbConn};
use error::{GreaseError, GreaseResult};
use extract::Extract;
use serde::{Serialize, Deserialize};

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
    pub conn: DbConn,
}

impl User {
    /// Checks if a member has a permission.
    pub fn has_permission(&self, permission_name: &str, event_type: Option<&str>) -> bool {
        let permission = MemberPermission {
            name: permission_name.to_owned(),
            event_type: event_type.map(|type_| type_.to_owned()),
        };

        self.permissions.contains(&permission)
    }
}

impl Extract for User {
    /// Extract a member from a request.
    ///
    /// Checks for a header named "token" containing the API token
    /// to authenticate a request as from the current user.
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        let mut conn = DbConn::extract(request)?;
        let member = request
            .headers()
            .get("token")
            .ok_or(GreaseError::Unauthorized)
            .and_then(|token_header| {
                token_header.to_str().map_err(|err| {
                    GreaseError::BadRequest(format!("invalid token header: {:?}", err))
                })
            })
            .and_then(|token| MemberForSemester::load_from_token(token, &mut conn))?;
        let permissions = member.permissions(&mut conn)?;

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
#[derive(PartialEq, Debug, Serialize, Deserialize, grease_derive::Extract)]
pub struct MemberPermission {
    pub name: String,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
}

impl Into<MemberPermission> for (String, Option<String>) {
    fn into(self) -> MemberPermission {
        MemberPermission {
            name: self.0,
            event_type: self.1,
        }
    }
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
            return Err(crate::error::GreaseError::Forbidden($permission.to_owned()));
        }
    };
}
