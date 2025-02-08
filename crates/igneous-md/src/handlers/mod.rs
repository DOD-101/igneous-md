//! Handler for incoming requests.
//!
//! Other than the initial setup and getting files from disk, all communication happens via
//! the [Websocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket).
//!
//! There is exactly one Websocket for every client.
//!
//! None of these functions should ever panic
use rocket::{fs::NamedFile, response::content::*, Request, State};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

/// Serve a css file from disk, from [Paths.config_dir]
///
/// The optional `_noise` parameter is used to force a reload of the css, by invalidating the browser cache.
#[get("/css/<file..>?<_noise>")]
pub async fn serve_css(
    file: PathBuf,
    _noise: Option<String>,
    paths: &State<Paths>,
) -> Option<NamedFile> {
    NamedFile::open(Path::new(&paths.get_css_dir()).join(&file))
        .await
        .ok()
}

/// Returns the initial html converted from the md file
///
/// This function only gets called the first time a client requests a markdown document,
/// any subsequent updates are handled via the websocket see [upgrade_connection()].
#[get("/?<css>", rank = 2)]
pub fn get_initial_md(css: Option<String>, paths: &State<Paths>) -> Option<RawHtml<String>> {
    let mut html = match fs::read_to_string(paths.get_default_md()) {
        Ok(md) => md_to_html(&md),
        Err(e) => {
            log::error!(
                "Failed to read .md file {}",
                paths.get_default_md().to_string_lossy()
            );
            log::trace!("{}", e);
            return None;
        }
    };

    html = initial_html(
        &css.map(|s| format!("css/{}", s))
            .unwrap_or(paths.get_default_css().to_string_lossy().to_string()),
        &html,
    );

    log::trace!("SERVER: Sending: {}", html);

    Some(RawHtml(html))
}
