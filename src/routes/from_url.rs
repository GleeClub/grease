use crate::error::{GreaseError, GreaseResult};
use std::collections::HashMap;
use std::str::FromStr;
use url::percent_encoding::percent_decode;
use url::Url;

pub trait FromUrlStr: Sized {
    fn from_url_str(in_str: &str) -> Option<Self>;
}

macro_rules! impl_from_url_str {
    ( $( $type:ty, )* ) => {
        $(
            impl FromUrlStr for $type {
                fn from_url_str(in_str: &str) -> Option<Self> {
                    <$type>::from_str(in_str).ok()
                }
            }
        )*
    };
}

impl_from_url_str!(usize, i32, i64, bool, String,);

impl<T: FromUrlStr> FromUrlStr for Option<T> {
    fn from_url_str(in_str: &str) -> Option<Self> {
        Some(T::from_url_str(in_str))
    }
}

pub fn parse_url(url: &str) -> GreaseResult<(Vec<String>, HashMap<String, String>)> {
    let url = Url::from_str(url).map_err(|err| {
        GreaseError::BadRequest(format!("couldn't parse url: {} (url = {})", err, url))
    })?;
    let segments = url
        .path_segments()
        .ok_or(GreaseError::BadRequest(
            "empty urls are not allowed".to_owned(),
        ))?
        .map(|segment| {
            percent_decode(segment.as_bytes())
                .decode_utf8_lossy()
                .to_string()
        })
        .collect();
    let params = url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    Ok((segments, params))
}
