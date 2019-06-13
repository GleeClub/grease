use crate::db::models::member::{MemberForSemester, MemberPermission};
use crate::error::{GreaseError, GreaseResult};
use crate::extract::Extract;
use diesel::mysql::MysqlConnection;


const TOKEN_NAME: &str = "GREASE_TOKEN";

pub struct User {
    pub member: MemberForSemester,
    pub permissions: Vec<MemberPermission>,
    pub conn: MysqlConnection,
}

// TODO: blanket impls with const generics?
// pub struct PermittedUser<const S: &'static str>(User);

impl Extract for User {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        let conn = <MysqlConnection as Extract>::extract(request)?;
        let member = request
            .headers()
            .get(TOKEN_NAME)
            .ok_or(GreaseError::Unauthorized)
            .and_then(|token| MemberForSemester::load_from_token(token.to_str().unwrap(), &conn))?;
        let permissions = member.permissions(&conn)?;

        Ok(User {
            member,
            permissions,
            conn,
        })
    }
}
