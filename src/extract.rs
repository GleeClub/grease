//! Deserialization of request data into usuable formats.
//!
//! The `grease_derive` sub-crate in this repository allows the
//! procedural derivation of the [Extract](crate::extract::Extract) trait
//! for anything that implements [Deserialize](serde::Deserialize).

use crate::error::GreaseResult;

/// The trait for automatic extraction of request data for
/// convenient endpoint consumption.
///
/// There are multiple blanket impls for extracting multiple things
/// from a request into a tuple for convenience.
///
/// This trait is specifically not implemented for the request itself
/// to force extraction to end at the endpoint argument level and simplify
/// endpoint bodies.
pub trait Extract: Sized {
    /// Try to extract Self from the request, and handle errors gracefully.
    fn extract(request: &cgi::Request) -> GreaseResult<Self>;
}

impl Extract for () {
    fn extract(_request: &cgi::Request) -> GreaseResult<Self> {
        Ok(())
    }
}

impl<T: Extract> Extract for GreaseResult<T> {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        Ok(T::extract(request))
    }
}

impl<T: Extract, U: Extract> Extract for (T, U) {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        Ok((T::extract(request)?, U::extract(request)?))
    }
}

impl<T: Extract, U: Extract, V: Extract> Extract for (T, U, V) {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        Ok((
            T::extract(request)?,
            U::extract(request)?,
            V::extract(request)?,
        ))
    }
}

impl<T: Extract, U: Extract, V: Extract, W: Extract> Extract for (T, U, V, W) {
    fn extract(request: &cgi::Request) -> GreaseResult<Self> {
        Ok((
            T::extract(request)?,
            U::extract(request)?,
            V::extract(request)?,
            W::extract(request)?,
        ))
    }
}
