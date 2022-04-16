use time::OffsetDateTime;

const HEADER_TOKEN: &'static str = "token";

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
