use async_graphql::{Context, Guard, Result};
use models::member::Member;

mod input;
mod models;
mod mutation;
mod permission;
mod query;

pub struct LoggedIn;

#[async_trait::async_trait]
impl Guard for LoggedIn {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Member>().is_some() {
            Ok(())
        } else {
            Err("User must be logged in".into())
        }
    }
}

pub fn handle_request(request: cgi::Request) -> cgi::Response {
    let mut response = None;
    let bt = Arc::new(Mutex::new(None));

    let bt2 = bt.clone();
    std::panic::set_hook(Box::new(move |_| {
        *bt2.lock().unwrap() = Some(Backtrace::new());
    }));

    panic::catch_unwind(AssertUnwindSafe(|| {
        if request.method() == "OPTIONS" {
            response = Some(options_response());
            return;
        }

        let (status_code, value) = match route_request(&request) {
            Ok(resp) => (200, resp),
            Err(error) => error.as_response(),
        };

        let body = serde_json::to_string(&value)
            .unwrap_or_default()
            .into_bytes();
        response = Some(
            response::Builder::new()
                .status(status_code)
                .header(CONTENT_TYPE, "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .header(CONTENT_LENGTH, body.len().to_string().as_str())
                .body(body)
                .unwrap(),
        );
    }))
    .ok();

    response.unwrap_or_else(move || {
        let error = bt
            .lock()
            .unwrap()
            .as_ref()
            .map(|bt| format!("{:?}", bt))
            .unwrap_or_default();
        crate::util::log_panic(&request, error)
    })
}
