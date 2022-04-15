const HEADER_TOKEN: &'static str = "token";

fn main() {
    dotenv::dotenv().ok();
    cgi::handle(|request| handle_request(request).unwrap_or_else(response::error));
}

pub fn handle_request(request: cgi::Request) -> Result<cgi::Response, String> {
    if request.method() == "OPTIONS" {
        return Ok(response::options());
    }

    ensure_member_is_webmaster(&request)?;

    let path = request
        .headers()
        .get("x-cgi-path-info")
        .map(|uri| uri.to_str().unwrap())
        .unwrap_or("/");

    let action = match path {
        "/upload_api" => actions::upload_api(request),
        "/run_migration" => actions::run_migration(),
        unknown => Err(format!(
            "The requested action (\"{}\") does not exist.",
            unknown
        )),
    };

    action.map(|_| response::success())
}

pub fn get_session_key_from_header<'a>(request: &'a cgi::Request) -> &'a str {
    request
        .headers()
        .get(HEADER_TOKEN)
        .and_then(|header| header.to_str().ok())
        .unwrap_or("")
}

pub fn ensure_member_is_webmaster(request: &cgi::Request) -> Result<(), String> {
    let session_key = get_session_key_from_header(request);
    let mut conn = db::get_conn()?;
    let member_positions = db::load_member_positions(session_key, &mut conn)?;

    if member_positions
        .iter()
        .any(|position| position.to_lowercase() == "webmaster")
    {
        Ok(())
    } else {
        Err("You must be a webmaster to use the admin tools.".to_owned())
    }
}

mod actions {
    use std::fs::write;
    use std::process::Command;

    const API_FILE_NAME: &'static str = "api";

    pub fn upload_api(request: cgi::Request) -> Result<(), String> {
        write(API_FILE_NAME, request.body())
            .map_err(|err| format!("Couldn't write file to disk: {:?}", err))?;

        Command::new("chmod")
            .args(&["+x", API_FILE_NAME])
            .output()
            .map_err(|err| {
                format!(
                    "Couldn't run `chmod` to make the new api executable: {:?}",
                    err
                )
            })
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err("`chmod` failed to make the new api executable.".to_owned())
                }
            })
    }

    pub fn run_migration() -> Result<(), String> {
        let migration_args = std::fs::read_to_string("/httpdocs/dev/smores/migration_command.txt")
            .map_err(|err| {
                format!(
                    "Couldn't retrieve passwords for the old and new databases: {:?}",
                    err
                )
            })?;

        match Command::new("/httpdocs/dev/smores/migration_script")
            .args(migration_args.trim().split_whitespace().skip(1))
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(err) => Err(format!(
                "Couldn't spawn the migration script as a child process: {:?}",
                err
            )),
        }
    }
}

mod db {
    use mysql::{self, Conn};

    const DB_URL_ENV_VAR: &'static str = "DATABASE_URL";

    pub fn get_conn() -> Result<Conn, String> {
        std::env::var(DB_URL_ENV_VAR)
            .map_err(|_| "Database url missing".to_owned())
            .and_then(|db_url| {
                Conn::new(db_url).map_err(|err| format!("database error: {:?}", err))
            })
    }

    pub fn load_member_positions(
        session_key: &str,
        conn: &mut Conn,
    ) -> Result<Vec<String>, String> {
        let query = format!(
            "
        SELECT r.`role`
        FROM member_role AS r
        WHERE r.`member` IN
            (SELECT s.`member`
             FROM session AS s
             WHERE s.`key` = \"{}\")
        ",
            session_key
        );

        conn.query(query)
            .and_then(|result| {
                result
                    .map(|row| row.map(|row| mysql::from_row_opt(row).unwrap_or("".to_owned())))
                    .collect::<Result<Vec<String>, _>>()
            })
            .map_err(|err| format!("querying error: {:?}", err))
    }
}

mod response {
    use cgi::http::header::CONTENT_TYPE;
    use cgi::http::response;

    pub fn success() -> cgi::Response {
        cgi::empty_response(204)
    }

    pub fn error(error_message: String) -> cgi::Response {
        response::Builder::new()
            .status(400)
            .header(CONTENT_TYPE, "application/json")
            .body(format!("{{\"error\":\"{}\"}}", &error_message).into())
            .unwrap()
    }

    pub fn options() -> cgi::Response {
        response::Builder::new()
            .status(204)
            .header("Allow", "POST, OPTIONS")
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header(
                "Access-Control-Allow-Headers",
                "token,access-control-allow-origin,content-type",
            )
            .body(Vec::new())
            .unwrap()
    }
}
