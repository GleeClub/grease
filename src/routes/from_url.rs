//! Handles the parsing of URL's for routing.

use crate::error::{GreaseError, GreaseResult};
use std::collections::HashMap;
use std::str::FromStr;
use url::percent_encoding::percent_decode;
use url::Url;

/// Parse the given type from a url string for routing.
pub trait FromUrlStr: Sized {
    /// Attempt to parse the given type from a string, and
    /// return None on failure.
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

            impl FromUrlStr for Option<$type> {
                fn from_url_str(in_str: &str) -> Option<Self> {
                    Some(<$type>::from_str(in_str).ok())
                }
            }
        )*
    };
}

impl_from_url_str!(usize, i32, i64, bool,);

impl FromUrlStr for String {
    fn from_url_str(in_str: &str) -> Option<Self> {
        Some(in_str.to_owned())
    }
}

impl FromUrlStr for Option<String> {
    fn from_url_str(in_str: &str) -> Option<Self> {
        Some(Some(in_str.to_owned()).filter(|s| s.len() > 0))
    }
}

/// Parse a url into its path segments and query parameters for routing convenience.
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
