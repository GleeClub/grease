use error::{GreaseError, GreaseResult};
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use std::fs::File;
use std::io::Read;

pub fn send_email(
    from_address: &str,
    to_email_list: &str,
    subject: &str,
    content: &str,
) -> GreaseResult<()> {
    let email = EmailBuilder::new()
        .to(to_email_list)
        .from(from_address)
        .subject(subject)
        .text(content)
        .build()
        .map_err(|err| GreaseError::ServerError(format!("error building the email: {:?}", err)))?;

    let mut mailer = SmtpClient::new_unencrypted_localhost()
        .map_err(|err| GreaseError::ServerError(format!("couldn't build mail client: {:?}", err)))?
        .transport();

    mailer
        .send(email.into())
        .map_err(|err| GreaseError::ServerError(format!("couldn't send email: {:?}", err)))?;

    Ok(())
}

pub fn random_base64(length: usize) -> GreaseResult<String> {
    let mut f = File::open("/dev/urandom")
        .map_err(|_err| GreaseError::ServerError("couldn't open /dev/urandom".to_owned()))?;

    std::iter::repeat_with(|| {
        let mut buffer: [u8; 1] = [0];
        f.read(&mut buffer).map_err(|err| {
            GreaseError::ServerError(format!("couldn't read /dev/urandom: {:?}", err))
        })?;
        Ok(buffer[0] as char)
    })
    .filter_map(|rand_char| match rand_char {
        Ok(c)
            if ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || ('0'..='9').contains(&c) =>
        {
            Some(Ok(c))
        }
        Ok(_bad_char) => None,
        Err(e) => Some(Err(e)),
    })
    .take(length)
    .collect()
}
