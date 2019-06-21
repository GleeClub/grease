use crate::db::models::member::{MemberForSemester, MemberPermission};
use crate::error::{GreaseError, GreaseResult};
use crate::routes::from_url::parse_url;
use extract::Extract;
use mysql::Conn;

pub struct User {
    pub member: MemberForSemester,
    pub permissions: Vec<MemberPermission>,
    pub conn: Conn,
}

// TODO: blanket impls with const generics?
// pub struct PermittedUser<const S: &'static str>(User);

impl User {
    pub fn has_permission(&self, permission_name: &str, event_type: Option<&str>) -> bool {
        let permission = MemberPermission {
            name: permission_name.to_owned(),
            event_type: event_type.map(|type_| type_.to_owned()),
        };

        self.permissions.contains(&permission)
    }
}

impl Extract for User {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        let mut conn = Conn::extract(request)?;
        let (_segments, params) = parse_url(&request.uri().to_string())?;
        let member = params
            .get("token")
            .ok_or(GreaseError::Unauthorized)
            .and_then(|token| MemberForSemester::load_from_token(token, &mut conn))?;
        let permissions = member.permissions(&mut conn)?;

        Ok(User {
            member,
            permissions,
            conn,
        })
    }
}
