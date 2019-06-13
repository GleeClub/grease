use crate::error::GreaseResult;

pub trait Extract: Sized {
    fn extract(request: &cgi::Request) -> GreaseResult<Self>;
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
