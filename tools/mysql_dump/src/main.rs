extern crate cgi;
extern crate chrono;
extern crate http;
extern crate mysql;
extern crate serde;
extern crate serde_json;
extern crate url;

use chrono::{Local, NaiveTime, TimeZone};
use http::{header, response};
use mysql as my;
use mysql::Value as SqlValue;
use serde_json::{json, to_value, Value};
use url::Url;

fn main() {
    cgi::handle(|request: cgi::Request| -> cgi::Response {
        let plain_url = match request
            .headers()
            .get("x-cgi-path-info")
            .map(|q| q.to_str().unwrap_or(""))
            .unwrap_or("")
        {
            path if path.len() > 1 => path[1..].to_string(),
            _short => {
                return cgi::string_response(
                    401,
                    "must pass DATABASE_URL with credentials as the path",
                )
            }
        };

        let url = match Url::parse(&plain_url) {
            Ok(mut url) => {
                url.set_path(&plain_url);
                format!("mysql://{}", url.path()[8..].to_string())
            }
            Err(error) => {
                return cgi::string_response(
                    401,
                    format!("DATABASE_URL was malformed: {:?}", error),
                )
            }
        };

        match retrieve_members(&url) {
            Ok(members_json) => {
                let body = members_json.to_string().into_bytes();
                response::Builder::new()
                    .status(200)
                    .header(header::CONTENT_LENGTH, body.len().to_string().as_str())
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(body)
                    .unwrap()
            }
            Err(message) => cgi::string_response(200, message),
        }
    })
}

fn retrieve_members(url: &str) -> Result<Value, String> {
    let pool = my::Pool::new(url).map_err(|err| format!("connection failed: {:?}", err))?;

    let tables: Vec<String> = pool
        .prep_exec(
            "SELECT table_name FROM information_schema.tables WHERE table_type = 'base table';",
            (),
        )
        .map_err(|err| format!("error getting table names: {:?}", err))
        .and_then(|result| {
            result
                .map(|row| {
                    let row = row.map_err(|err| format!("couldn't read row: {:?}", err))?;
                    row.get(0)
                        .ok_or("Error retrieving table name from row".to_owned())
                })
                .collect::<Result<Vec<String>, String>>()
        })?;

    let mut out = json!({});

    for table_name in tables {
        let values = pool
            .prep_exec(&format!("SELECT * from {}", table_name), ())
            .map_err(|err| format!("exec error: {:?}", err))
            .and_then(|result| {
                result
                    .map(|row| {
                        let mut row = row.map_err(|err| format!("couldn't read row: {:?}", err))?;

                        let mut out = json!({});
                        for (ind, column) in row.columns().iter().enumerate() {
                            let val = match row.take::<SqlValue, _>(ind).ok_or(format!(
                                "Couldn't load data from row in table {} at index {}",
                                table_name, ind
                            ))? {
                                SqlValue::NULL => Value::Null,
                                SqlValue::Bytes(bytes) => Value::String(
                                    std::str::from_utf8(&bytes)
                                        .map_err(|err| {
                                            format!("error deserializing into string: {:?}", err)
                                        })?
                                        .to_string(),
                                ),
                                SqlValue::Int(int) => json!(int),
                                SqlValue::UInt(uint) => json!(uint),
                                SqlValue::Float(float) => json!(float),
                                SqlValue::Date(year, month, day, hour, minutes, seconds, micro) => {
                                    let datetime = Local
                                        .ymd(year as i32, month as u32, day as u32)
                                        .and_hms_micro(
                                            hour as u32,
                                            minutes as u32,
                                            seconds as u32,
                                            micro,
                                        );
                                    Value::String(format!("{}", datetime.format("%+")))
                                }
                                SqlValue::Time(
                                    is_negative,
                                    days,
                                    hours,
                                    minutes,
                                    seconds,
                                    micro,
                                ) => {
                                    let time = NaiveTime::from_hms_micro(
                                        24 * days + hours as u32,
                                        minutes as u32,
                                        seconds as u32,
                                        micro,
                                    );
                                    Value::String(format!(
                                        "{}{}",
                                        if is_negative { "-" } else { "" },
                                        time.format("%H:%M:%S%.6f")
                                    ))
                                }
                            };
                            out[column.name_str().to_string()] = to_value(val)
                                .map_err(|err| format!("couldn't parse row as json: {:?}", err))?;
                        }
                        Ok(out)
                        // to_value(format!("{:?}", row)).map_err(|err| format!("error formatting to json: {:?}", err))
                    })
                    .collect::<Result<Vec<Value>, String>>()
            })?;
        out[table_name] = to_value(values).map_err(|err| {
            format!(
                "error converting values from table {}: {:?}",
                table_name, err
            )
        })?;
    }

    Ok(out)
}
