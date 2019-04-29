pub struct GreaseError {
    status_code: usize,
    content: String,
}

impl GreaseError {
    pub fn with_code(code: usize) -> Self {
        unimplemented!()
    }

    fn page_for_code(code: usize) -> Option<String> {
        unimplemented!()
    }

    pub fn bad_request_message(reason: &str) -> Self {
        unimplemented!()
    }

    pub fn forbidden() -> Self {
        unimplemented!()
    }

    pub fn unauthorized() -> Self {
        unimplemented!()
    }
}
