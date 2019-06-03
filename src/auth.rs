use diesel::mysql::MysqlConnection;
use crate::db::models::Member;
use crate::extract::Extract;
use crate::error::{GreaseError, GreaseResult};

const TOKEN_NAME: &str = "GREASE_TOKEN";

pub struct User {
    member: Member,
    permissions: Vec<String>,
    conn: MysqlConnection,
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
            .and_then(|token| Member::load_from_token(token.to_str().unwrap(), &conn)
                .transpose()
                .unwrap_or(Err(GreaseError::Unauthorized))
            )?;
        let permissions = member.permissions(&conn)?;

        Ok(User {
            member,
            permissions,
            conn,
        }) 
    }
}
