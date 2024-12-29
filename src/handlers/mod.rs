//! Functions for handling incoming requests
//!
//! None of these functions should ever panic
use rocket::{http::Status, response::content::*, State};
use std::{fs, sync::Mutex};

use crate::{config::Config, config_path, convert::initial_html, convert::md_to_html};

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
pub fn get_inital_md(path: &str, config: &State<Mutex<Config>>) -> Option<RawHtml<String>> {
    let mut html = match fs::read_to_string(path) {
        Ok(md) => md_to_html(&md),
        Err(e) => {
            log::error!("Failed to read .md file {}", path);
            log::trace!("{}", e);
            return None;
        }
    };

    let config = config.lock().unwrap();

    html = initial_html(
        &config.current_css().unwrap_or("".into()).to_string_lossy(),
        &html,
    );

    log::trace!("SERVER: Sending: {}", html);

    Some(RawHtml(html))
}

/// Saves the given html string to disk
///
/// The file is stored in the users config dir, with the name:
/// `html-export-<year>-<month>-<day>-<hour>-<minute>-<second>.html`
///
/// It is possible that one file overwrites another if the user happens to press the export button
/// twice in one second, but this should never happen in normal use.
#[post("/api/post-html", data = "<body_data>")]
pub async fn save_html(body_data: String) -> Result<(), Status> {
    // Save the HTML string to a file
    let file_path = format!(
        "{}html-export-{}.html",
        config_path(),
        chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
    );

    if std::fs::write(&file_path, body_data).is_err() {
        return Err(Status::InternalServerError); // Handle file save errors
    }

    log::info!("Exported HTML to: {}", file_path);

    Ok(())
}
