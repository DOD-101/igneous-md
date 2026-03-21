//! Handler for incoming requests.
//!
//! Other than the initial setup and getting files from disk, all communication happens via
//! the [Websocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
//!
//! There is exactly one Websocket for every client.
//!
//! None of these functions should ever panic
use rocket::{response::content::*, Request};

mod ws;
pub use ws::upgrade_connection;

#[catch(404)]
pub fn not_found() -> &'static str {
    "404 Not Found"
}

#[catch(500)]
pub fn internal_error(req: &Request) -> RawHtml<String> {
    RawHtml(format!(
        r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <title>500 Internal Server Error</title>
    </head>
    <body>
        <h1>500 Internal Server Error</h1>
        <p> The request that caused this was: </p>
        <code style="display=block;">{:#?}</code>
        <p>This is a bug, please report it at <a href="https://github.com/dod-101/igneous-md/issues">https://github.com/dod-101/igneous-md/issues</a> making sure to include the above debug information.</p>
    </body>
</html>
"#,
        req
    ))
}
