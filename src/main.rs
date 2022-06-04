//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website

use std::process::Command;

use anyhow::{bail, Context, Result};
use cgi::http::Method;
use clap::Parser;
use grease::db::DbConn;
use grease::graphql::build_schema;
use grease::models::member::Member;
use grease::models::permissions::MemberRole;
use grease::util::{get_token_from_header, gql_err_to_anyhow, options_response};

const API_FILE_NAME: &str = "api";

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<SubCommand>,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    Api,
    Schema,
    Migrate,
    SendEmails,
    Upload,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match args.command.unwrap_or(SubCommand::Api) {
        SubCommand::Api => {
            cgi::handle(|request| {
                rt.block_on(async move {
                    match grease::graphql::handle(request).await {
                        Ok(response) => response,
                        Err(error) => cgi::text_response(500, error.to_string()),
                    }
                })
            });
        }
        SubCommand::Schema => println!("{}", build_schema().sdl()),
        SubCommand::Migrate => rt.block_on(run_migrations())?,
        SubCommand::SendEmails => rt.block_on(async move {
            let conn = grease::db::DbConn::connect().await?;
            grease::email::send_emails(&conn).await
        })?,
        SubCommand::Upload => {
            cgi::handle(|request| {
                rt.block_on(async move {
                    let result = ensure_member_is_webmaster(&request)
                        .await
                        .and_then(|()| upload_api(request));
                    match result {
                        Ok(()) => cgi::empty_response(204),
                        Err(error) => cgi::text_response(500, error.to_string()),
                    }
                })
            });
        }
    }

    Ok(())
}

pub async fn handle_request(request: cgi::Request) -> Result<cgi::Response> {
    if request.method() == Method::OPTIONS {
        return Ok(options_response());
    }

    ensure_member_is_webmaster(&request).await?;

    match request.uri().path() {
        "/upload-api" => upload_api(request)?,
        "/migrate" => run_migrations().await?,
        // "/restore-db" => restore_db(request).await?,
        // "/backup-db" => return backup_db().await,
        unknown => bail!("The requested action \"{}\" does not exist.", unknown),
    }

    Ok(cgi::empty_response(204))
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
