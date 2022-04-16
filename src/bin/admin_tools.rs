//! # Admin Tools
//!
//! Tools for managing the

use std::process::Command;
use anyhow::{bail, Context, Result};
use cgi::http::response::Builder;
use cgi::http::Method;
use grease::db::DbConn;
use grease::models::member::Member;
use grease::models::permissions::MemberRole;
use grease::util::{get_token_from_header, gql_err_to_anyhow};

const API_FILE_NAME: &'static str = "api";

fn main() {
    dotenv::dotenv().ok();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    cgi::handle(|request| {
        rt.block_on(async move { cgi::err_to_500(handle_request(request).await) })
    });
}

pub async fn handle_request(request: cgi::Request) -> Result<cgi::Response> {
    if request.method() == Method::OPTIONS {
        return Ok(options());
    }

    ensure_member_is_webmaster(&request).await?;

    match request.uri().path() {
        "/upload-api" => upload_api(request)?,
        "/migrate" => run_migrations().await?,
        unknown => bail!("The requested action \"{}\" does not exist.", unknown),
    }

    Ok(cgi::empty_response(204))
}

pub fn options() -> cgi::Response {
    Builder::new()
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

pub async fn ensure_member_is_webmaster(request: &cgi::Request) -> Result<()> {
    let conn = DbConn::connect().await?;
    let token = get_token_from_header(request).context("No token in header")?;
    let member = Member::with_token(token, &conn)
        .await
        .map_err(gql_err_to_anyhow)?;

    if MemberRole::member_has_role(&member.email, "Webmaster", &conn)
        .await
        .map_err(gql_err_to_anyhow)?
    {
        Ok(())
    } else {
        anyhow::bail!("You must be a webmaster to use this tool")
    }
}

pub fn upload_api(request: cgi::Request) -> Result<()> {
    std::fs::write(API_FILE_NAME, request.body()).context("Couldn't write file to disk")?;

    let chmod_output = Command::new("chmod")
        .args(&["+x", API_FILE_NAME])
        .output()
        .context("Couldn't run `chmod` to make the new api executable")?;

    if chmod_output.status.success() {
        Ok(())
    } else {
        bail!("Failed to make the new api executable")
    }
}

pub async fn run_migrations() -> Result<()> {
    let mut conn = DbConn::connect().await?.into_inner();

    sqlx::migrate!()
        .run(&mut conn)
        .await
        .context("Failed to run database migrations")
}
