//! Functions for handling incoming requests
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

#[get("/src/main.js")]
pub fn serve_main_js() -> RawJavaScript<&'static str> {
    RawJavaScript(include_str!("../main.js"))
}

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
