//! The `router` macro for routing requests to endpoints.

/// Route requests to endpoints.
///
/// The macro is strongly inspired by (a.k.a. ripped out of and slightly modified
/// from) the `router` from [rouille](https://crates.io/crates/rouille). It is similar
/// to a match expression for endpoints, in that it requires path parameters and query
/// parameters of specific types, and if the endpoint matches, the extracted parameters
/// are passed to a given endpoint. The general structure is as follows:
///
/// ```rust
/// router!(request,
///     (GET)  [/foo/(id: i32)?(full: Option<bool>)] => get_foo,
///     (POST) [/foo/(id: i32)] => update_foo,
/// )
/// ```
///
/// Where
/// ```rust
/// get_foo: Fn(i32, Option<bool>, Extract) -> GreaseResult<Value>
/// ```
/// and
/// ```rust
/// update_foo: Fn(i32, Extract) -> GreaseResult<Value>
/// ```
///
/// The above shows that (GET) requires a specific type of HTTP method, and
/// [/foo/(id: i32)?(full: Option<bool>)] requires requests along the lines of
/// "/foo/3?full=true" or "/foo/6" (an i32 "id" is required and a "full" query
/// parameter is optionally accepted).
///
/// The routes are processed in the order written, and the first one matched has
/// the callback called with the extracted parameters. If none are matched, an
/// `Err(GreaseError::NotFound)` is returned.
///
/// Note that this doesn't check for duplicate routes, so be careful to not add them
/// as to avoid unnecessary parse time in request handling.
#[macro_export]
macro_rules! router {
    ($request:expr, $( ($method:ident) [ $( $path:tt )* ] => $callback:expr, )* ) => {{
        let (segments, params) = crate::routes::router::parse_url($request)?;

        $({
            if $request.method() == stringify!($method) {
                let mut segment_iter = segments.iter();
                router!(@check segment_iter, params, $callback, (), $( $path )*);
            }
        })*

        Err(GreaseError::NotFound)
    }};
    // segment parse check
    (@check $segment_iter:ident, $params:expr, $callback:expr, ( $( $f_args:expr, )* ), /($p:ident: $t:ty) $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if let Some($p) = next_segment.parse::<$t>().ok() {
                router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $( $rest )*);
            }
        }
    );
    // query param check
    (@check $segment_iter:ident, $params:expr, $callback:expr, ( $( $f_args:expr, )* ), ?($p:ident: $t:ty) $($rest:tt)*) => (
        {
            let $p = $params.get(stringify!($p)).and_then(|p| p.parse::<$t>().ok());
            router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $( $rest )*);
        }
    );
    // plain segment check
    (@check $segment_iter:ident, $params:expr, $callback:expr, ( $( $f_args:expr, )* ), /$p:ident $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if next_segment == stringify!($p) {
                router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* ), $( $rest )*);
            }
        }
    );
    // terminal check
    (@check $segment_iter:ident, $params:expr, $callback:expr, ( $( $f_args:expr, )* ), ) => (
        if $segment_iter.next().is_none() {
            return ($callback)( $( $f_args, )* );
        }
    );
}

/// Parse a url into its path segments and query parameters for routing convenience.
pub fn parse_url(
    request: &cgi::Request,
) -> crate::error::GreaseResult<(Vec<String>, std::collections::HashMap<String, String>)> {
    use error::GreaseError;
    use std::str::FromStr as _;
    use url::{
        percent_encoding::{percent_decode, utf8_percent_encode, DEFAULT_ENCODE_SET},
        Url,
    };

    let path = request
        .headers()
        .get("x-cgi-path-info")
        .and_then(|uri| uri.to_str().ok())
        .unwrap_or("/");
    let param_str = request
        .headers()
        .get("x-cgi-query-string")
        .and_then(|uri| uri.to_str().ok())
        .unwrap_or("");

    let given_url = format!(
        "https://gleeclub.gatech.edu{}?{}",
        utf8_percent_encode(&path, DEFAULT_ENCODE_SET).to_string(),
        utf8_percent_encode(&param_str, DEFAULT_ENCODE_SET).to_string()
    );
    let given_url = Url::from_str(&given_url).map_err(|err| {
        GreaseError::BadRequest(format!("couldn't parse url: {} (url = {})", err, given_url))
    })?;
    let segments = given_url
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
    let params = given_url
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    Ok((segments, params))
}
