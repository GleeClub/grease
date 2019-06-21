use error::{GreaseError, GreaseResult};
// use lettre::{SmtpClient, Transport};
// use lettre_email::EmailBuilder;
use base64::decode;
use chrono::{Local, NaiveDateTime};
use glob::glob;
use serde::Deserialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

// pub fn send_email(
//     from_address: &str,
//     to_email_list: &str,
//     subject: &str,
//     content: &str,
// ) -> GreaseResult<()> {
//     let email = EmailBuilder::new()
//         .to(to_email_list)
//         .from(from_address)
//         .subject(subject)
//         .text(content)
//         .build()
//         .map_err(|err| GreaseError::ServerError(format!("error building the email: {:?}", err)))?;

//     let mut mailer = SmtpClient::new_unencrypted_localhost()
//         .map_err(|err| GreaseError::ServerError(format!("couldn't build mail client: {:?}", err)))?
//         .transport();

//     mailer
//         .send(email.into())
//         .map_err(|err| GreaseError::ServerError(format!("couldn't send email: {:?}", err)))?;

//     Ok(())
// }

#[derive(Deserialize, grease_derive::Extract)]
pub struct FileUpload {
    pub path: String,
    pub content: String,
}

impl FileUpload {
    pub fn upload(&self) -> GreaseResult<()> {
        let content = decode(&self.content).map_err(|err| {
            GreaseError::BadRequest(format!("couldn't decode file as base64: {}", err))
        })?;
        let path = {
            let given_path = PathBuf::from_str(&self.path).map_err(|_err| {
                GreaseError::BadRequest(format!("invalid file name: {}", &self.path))
            })?;
            let file_name = given_path.file_name().ok_or(GreaseError::BadRequest(
                "file name must end in an absolute path".to_owned(),
            ))?;
            let _extension = given_path.extension().ok_or(GreaseError::BadRequest(
                "file must have an extension".to_owned(),
            ))?;
            let mut base_path = PathBuf::from("./music/");
            base_path.push(file_name);

            base_path
        };
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|err| GreaseError::ServerError(format!("error opening file: {}", err)))?;
        file.write_all(&content)
            .map_err(|err| GreaseError::ServerError(format!("error writing to file: {}", err)))?;

        Ok(())
    }
}

pub fn check_for_music_file(path: &str) -> GreaseResult<String> {
    let given_path = PathBuf::from_str(path)
        .map_err(|_err| GreaseError::BadRequest(format!("invalid file name: {}", path)))?;
    let file_name = given_path
        .file_name()
        .ok_or(GreaseError::BadRequest(
            "file name must end in an absolute path".to_owned(),
        ))?
        .to_string_lossy()
        .to_string();
    let _extension = given_path.extension().ok_or(GreaseError::BadRequest(
        "file must have an extension".to_owned(),
    ))?;

    let mut existing_path = PathBuf::from("./music/");
    existing_path.push(&file_name);

    if std::fs::metadata(existing_path).is_ok() {
        Ok(file_name)
    } else {
        Err(GreaseError::BadRequest(format!(
            "the file {} doesn't exist yet and must be uploaded before a link to it can be made",
            file_name
        )))
    }
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

pub fn log_panic(request: &cgi::Request, error_message: String) -> cgi::Response {
    let now = Local::now().naive_local();
    let file_name = format!("/cgi-bin/log/log {}.txt", now.format("%c"));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(file_name)
        .expect("couldn't open new log file");
    let write_to_file = |file: &mut std::fs::File, content: String| {
        file.write_all(content.as_bytes())
            .expect("couldn't write to log file");
    };

    let env_vars = std::env::vars().collect::<Vec<_>>();
    write_to_file(
        &mut file,
        format!(
            "At {}, panicked during request handling.\n",
            now.format("%c")
        ),
    );
    write_to_file(
        &mut file,
        format!("Enviroment variables:\n  {:?}\n", env_vars),
    );
    write_to_file(&mut file, format!("Request:\n  {:?}\n", request));
    write_to_file(
        &mut file,
        format!("Error generated:\n  {}\n", error_message),
    );

    clean_up_old_logs();

    let json_val = serde_json::json!({
        "message": "Panicked during handling of request. Please contact an administrator with the following information:",
        "time": now.format("%c").to_string(),
        "environment_variables": format!("{:?}", env_vars),
        "request": format!("{:?}", request),
        "error": error_message,
    });
    let body = json_val.to_string().into_bytes();

    http::response::Builder::new()
        .status(500)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            http::header::CONTENT_LENGTH,
            body.len().to_string().as_str(),
        )
        .body(body)
        .unwrap()
}

fn clean_up_old_logs() {
    let log_files: Vec<PathBuf> = glob("./log/*.txt")
        .expect("Failed to read glob pattern")
        .collect::<Result<Vec<_>, _>>()
        .expect("one of the log files had an invalid name");
    if log_files.len() >= 50 {
        let mut log_times = log_files
            .iter()
            .map(|log_file: &PathBuf| {
                let file_name = log_file
                    .file_name()
                    .expect("no file name found for log file")
                    .to_string_lossy();
                let time = NaiveDateTime::parse_from_str(&file_name, "log %c")
                    .expect("log file was incorrectly named");
                (log_file, time)
            })
            .collect::<Vec<(&PathBuf, NaiveDateTime)>>();
        log_times.sort_by_key(|(_log_file, time)| time.clone());

        log_times.iter().skip(49).for_each(|(log_file, _time)| {
            std::fs::remove_file(log_file).expect("could not delete old log file");
        });
    }
}
