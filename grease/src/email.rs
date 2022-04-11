//! Extra utilties for use elsewhere in the API.

use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub struct Email {
    pub to_address: String,
    pub subject: String,
    pub content: String,
}

impl Email {
    pub const DEFAULT_NAME: &'static str = "Glee Club Officers";
    pub const DEFAULT_ADDRESS: &'static str = "gleeclub_officers@lists.gatech.edu";

    pub fn send(&self) -> Result<()> {
        let mut mail = Command::new("mail")
            .args(&["-s", &self.subject, &self.to_address])
            .stdin(Stdio::piped())
            .spawn()
            .context("Couldn't run `mail` to send an email")?;

        let stdin = mail
            .stdin
            .as_mut()
            .context("No stdin was available for `mail`")?;
        stdin
            .write_all(self.content.as_bytes())
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
                String::from_utf8_lossy(output.stderr),
            ))
        }
    }
}
