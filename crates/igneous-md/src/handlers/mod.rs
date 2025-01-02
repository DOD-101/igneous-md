//! Handler for incoming requests.
//!
//! Other than the initial setup and getting files from disk, all communication happens via
//! the [Websocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
//!
//! There is exactly one Websocket for every client.
//!
//! None of these functions should ever panic
use rocket::{response::content::*, State};
use std::fs;

use crate::{
    convert::{initial_html, md_to_html},
    paths::Paths,
};

mod ws;
pub use ws::upgrade_connection;

/// Serve /src/main.js from a string included in the binary at compile time
///
/// ## Note:
///
/// This is done so that igneous-md can be installed via `cargo install` since, cargo cannot
/// install any file other than a single binary. Hence it is not possible for us to serve this
/// statically as we would normally do from disk.
#[get("/src/main.js")]
pub fn serve_main_js() -> RawJavaScript<&'static str> {
    RawJavaScript(include_str!("../main.js"))
}

/// Serve /src/highlight.min.js from a string included in the binary at compile time
///
/// ## Note:
///
/// See [serve_main_js()]
#[get("/src/highlight.min.js")]
pub fn serve_highlight_js() -> RawJavaScript<&'static str> {
    RawJavaScript(include_str!("../highlight.min.js"))
}

/// Returns the initial html converted from the md file
///
/// This function only gets called the first time a client requests a markdown document,
/// any subsequent updates are handled via the websocket see `upgrade_connection`.
///
#[get("/?<path>", rank = 2)]
pub fn get_initial_md(path: &str, paths: &State<Paths>) -> Option<RawHtml<String>> {
    let mut html = match fs::read_to_string(path) {
        Ok(md) => md_to_html(&md),
        Err(e) => {
            log::error!("Failed to read .md file {}", path);
            log::trace!("{}", e);
            return None;
        }
    };

    html = initial_html(&paths.get_default_css().to_string_lossy(), &html);

    log::trace!("SERVER: Sending: {}", html);

    Some(RawHtml(html))
}
