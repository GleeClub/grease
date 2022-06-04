use cgi::http::response::Builder;
use time::OffsetDateTime;

const HEADER_TOKEN: &str = "GREASE_TOKEN";

pub fn now() -> OffsetDateTime {
    OffsetDateTime::try_now_local().expect("Failed to get system time UTC offset")
}

pub fn get_token_from_header<'a>(request: &'a cgi::Request) -> Option<&'a str> {
    request
        .headers()
        .get(HEADER_TOKEN)
        .and_then(|header| header.to_str().ok())
}

pub fn gql_err_to_anyhow(err: async_graphql::Error) -> anyhow::Error {
    anyhow::anyhow!("{}", err.message)
}

pub fn options_response() -> cgi::Response {
    Builder::new()
        .status(204)
        .header("Allow", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "token,access-control-allow-origin,content-type",
        )
        .body(Vec::new())
        .unwrap()
}
