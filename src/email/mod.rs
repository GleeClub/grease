//! Extra utilties for use elsewhere in the API.

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub mod event;
pub mod minutes;

pub const MEMBER_LIST_ADDRESS: &'static str = "gleeclub@lists.gatech.edu";
pub const OFFICER_LIST_ADDRESS: &'static str = "gleeclub@lists.gatech.edu";
pub const FROM_ADDRESS: &'static str = "Glee Club Officers";

pub struct Email<'a> {
    pub address: &'a str,
    pub subject: String,
    pub body: String,
}

impl<'a> Email<'a> {
    pub async fn send(&'a self) -> Result<()> {
        let mut mail = Command::new("mail")
            .args(&["-s", &self.subject, &self.address])
            .stdin(Stdio::piped())
            .spawn()
            .context("Couldn't run `mail` to send an email")?;

        let stdin = mail
            .stdin
            .as_mut()
            .context("No stdin was available for `mail`")?;
        stdin
            .write_all(self.body.as_bytes())
            .context("Couldn't send an email with `mail`")?;

        let output = mail
            .wait_with_output()
            .context("The output of the `mail` command couldn't be retrieved")?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "`mail` failed to send an email with error code {}: {}",
                output.status.code().unwrap_or(1),
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }
}
