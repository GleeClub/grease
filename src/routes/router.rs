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
/// as to avoid unnecessary waits to request handling.
#[macro_export]
macro_rules! router {
    ($request:expr, $( ($method:ident) [ $( $path:tt )* ] => $callback:ident, )* ) => {{
        use crate::routes::from_url::{FromUrlStr, parse_url};
        use crate::extract::Extract;

        let (segments, params) = parse_url(&$request.uri().to_string())?;

        $({
            if $request.method() == stringify!($method) {
                let mut segment_iter = segments.iter();
                router!(@check segment_iter, params, $callback, (), Extract::extract($request), $( $path )*);
            }
        })*

        Err(GreaseError::NotFound)
    }};
    // segment parse check
    (@check $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, /($p:ident: $t:ty) $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if let Some($p) = <$t>::from_url_str(&next_segment) {
                router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $extract, $( $rest )*);
            }
        }
    );
    // query param check
    (@check $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, ?($p:ident: $t:ty) $($rest:tt)*) => (
        if let Some($p) = <$t>::from_url_str($params.get(stringify!($p)).unwrap_or(&"".to_owned())) {
            router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $extract, $( $rest )*);
        }
    );
    // plain segment check
    (@check $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, /$p:ident $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if next_segment == stringify!($p) {
                router!(@check $segment_iter, $params, $callback, ( $( $f_args, )* ), $extract, $( $rest )*);
            }
        }
    );
    // terminal check
    (@check $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, ) => (
        if $segment_iter.next().is_none() {
            return $callback( $( $f_args, )* $extract? );
        }
    );
}
