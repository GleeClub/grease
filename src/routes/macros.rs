#[macro_export]
macro_rules! router {
    ($request:expr, $( ($method:ident) [ $( $path:tt )* ] => $callback:ident, )* ) => {{
        use crate::routes::from_url::{FromUrlStr, parse_url};
        use crate::extract::Extract;

        let (segments, params) = parse_url(&$request.uri().to_string())?;

        $({
            if $request.method() == stringify!($method) {
                let mut segment_iter = segments.iter();
                router!(__check_pattern segment_iter, params, $callback, (), Extract::extract($request), $( $path )*);
            }
        })*

        Err(GreaseError::NotFound)
    }};
    // segment parse check
    (__check_pattern $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, /($p:ident: $t:ty) $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if let Some($p) = <$t>::from_url_str(&next_segment) {
                router!(__check_pattern $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $extract, $( $rest )*);
            }
        }
    );
    // query param check
    (__check_pattern $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, ?($p:ident: $t:ty) $($rest:tt)*) => (
        if let Some($p) = <$t>::from_url_str($params.get(stringify!($p)).unwrap_or(&"".to_owned())) {
            router!(__check_pattern $segment_iter, $params, $callback, ( $( $f_args, )* $p, ), $extract, $( $rest )*);
        }
    );
    // plain segment check
    (__check_pattern $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, /$p:ident $($rest:tt)*) => (
        if let Some(next_segment) = $segment_iter.next() {
            if next_segment == stringify!($p) {
                router!(__check_pattern $segment_iter, $params, $callback, ( $( $f_args, )* ), $extract, $( $rest )*);
            }
        }
    );
    // terminal check
    (__check_pattern $segment_iter:ident, $params:expr, $callback:ident, ( $( $f_args:expr, )* ), $extract:expr, ) => (
        if $segment_iter.next().is_none() {
            return $callback( $( $f_args, )* $extract? );
        }
    );
}
