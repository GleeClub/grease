use db::{
    member::{MemberForSemester, MemberPermission},
    DbConn,
};
use error::{GreaseError, GreaseResult};
use extract::Extract;

pub struct User {
    pub member: MemberForSemester,
    pub permissions: Vec<MemberPermission>,
    pub conn: DbConn,
}

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
