//! Functions for handling incoming requests
//!
//! None of these functions should ever panic

use kuchikiki::traits::*;
use markdown::{to_html_with_options, Options};
use rouille::{
    match_assets, try_or_400,
    websocket::{self, SendError, Websocket},
    Request, Response,
};
use std::{
    fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    process::exit,
    thread,
    time::{Duration, SystemTime},
};

use crate::{config::Config, config_path};

/// Gets the next style-sheet from the given Config
pub fn get_next_css_path(config: &mut Config) -> Response {
    match config.next_css() {
        Some(css) => Response::text(css.to_string_lossy()),
        None => Response::html("No css files found").with_status_code(501),
    }
}

/// Gets the previous style-sheet from the given Config
pub fn get_prev_css_path(config: &mut Config) -> Response {
    match config.previous_css() {
        Some(css) => Response::text(css.to_string_lossy()),
        None => Response::html("No css files found").with_status_code(501),
    }
}

/// Returns the requested css file if it exists
pub fn get_css(request: &Request, css_dir: &str) -> Response {
    let response = match_assets(&request.remove_prefix("/css").unwrap(), css_dir);

    if response.is_error() {
        log::warn!("Failed to match css: {}", request.url());
    }

    response
}

/// Returns the initial html converted from the md file
///
/// This function only gets called the first time a client requests a markdown document,
/// any subsequent updates are handled via the websocket see `upgrade_connection`.
pub fn get_inital_md(request: &Request, initial_css: &str) -> Response {
    // WARN: This seems like it might cause problems. This would also be
    // a good place for using Path rather than strings
    let mut html = match fs::read_to_string(format!(".{}", request.url())) {
        Ok(md) => md_to_html(&md),
        Err(e) => {
            log::error!("Failed to read .md file {}", request.url());
            log::trace!("{}", e);
            return Response::html("404 Error").with_status_code(404);
        }
    };

    html = initial_html(initial_css, &html);

    log::trace!("SERVER: Sending: {}", html);

    Response::html(html)
}

/// Handles clients upgrading to websocket to receive file updates
///
/// This function will upgrade the connection to websocket and spawn a new thread for the
/// connection.
pub fn upgrade_connection(request: &Request) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(request, Some("md-data")));

    // Get's the path from the arguments passed via the url
    // The name of the argument is ignored entirely
    let file_path =
        PathBuf::from(&request.raw_query_string().split('=').collect::<Vec<_>>()[1].to_string());

    if !file_path.exists() {
        log::warn!(
            "Failed to upgrade to websocket connection: {} Doesn't exist.",
            file_path.to_string_lossy()
        );
        return Response::html(format!(
            "404 Error: File doesn't exist: {}",
            file_path.to_string_lossy()
        ))
        .with_status_code(404);
    }
    if file_path.extension().unwrap_or_default() != "md" {
        log::warn!(
            "Failed to upgrade to websocket connection: {} Is not a .md file.",
            file_path.to_string_lossy()
        );
        return Response::html(format!(
            "404 Error: File {} isn't a .md file.",
            file_path.to_string_lossy()
        ))
        .with_status_code(404);
    }

    thread::spawn(move || {
        // Wait until the websocket is established
        let ws = match websocket.recv() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to establish websocket connection: {}", e);
                return;
            }
        };

        ws_update_md(ws, &file_path)
    });
    response
}

/// Saves the given html string to disk
///
/// The file is stored in the users config dir, with the name:
/// `html-export-<year>-<month>-<day>-<hour>-<minute>-<second>.html`
///
/// It is possible that one file overwrites another if the user happens to press the export button
/// twice in one second, but this should never happen in normal use.
pub fn save_html(request: &Request) -> Response {
    match request.data() {
        Some(mut html) => {
            let file_path = format!(
                "{}html-export-{}.html",
                config_path(),
                chrono::Local::now().format("%y-%m-%d-%H-%M-%S"),
            );
            // Convert request.data into a String
            let mut html_string: String = String::new();
            if html.read_to_string(&mut html_string).is_err() {
                return Response::html("500 Error: Failed to read html.").with_status_code(500);
            };

            if fs::write(&file_path, html_string).is_err() {
                return Response::html("500 Error: Failed to save to file.").with_status_code(500);
            };

            log::info!("Exported html to: {}", file_path);

            Response::html("Ok").with_status_code(200)
        }
        None => Response::html("404 Error").with_status_code(404),
    }
}
/// Internal logic for the websocket
///
/// Checks the metadata and every time it detects a file change will send the new markdown body to
/// the client.
fn ws_update_md(mut websocket: Websocket, file_path: &Path) {
    let mut last_modified = SystemTime::UNIX_EPOCH;
    loop {
        // Check if file has been modified
        let modified = match fs::metadata(file_path) {
            Ok(m) => match m.modified() {
                Ok(c) => c,
                Err(e) => {
                    log::error!(
                        "Error while checking if file: {} has been modified.",
                        file_path.to_string_lossy(),
                    );
                    log::trace!("{}", e);
                    exit(1)
                }
            },
            Err(e) => {
                log::error!(
                    "Failed to get file: {} metadata.",
                    file_path.to_string_lossy(),
                );
                log::trace!("{}", e);
                exit(1)
            }
        };

        if modified != last_modified {
            last_modified = modified;

            let file_contents = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!(
                        "Failed to read file: {} to string.",
                        file_path.to_string_lossy(),
                    );
                    log::trace!("{}", e);
                    exit(1)
                }
            };

            let html = md_to_html(&file_contents);

            log::trace!("SERVER: Sending: {html}");

            match websocket.send_text(&html) {
                Ok(_) => (),
                Err(SendError::Closed) => exit(0),
                Err(SendError::IoError(e)) if e.kind() == ErrorKind::BrokenPipe => {
                    log::info!("Websocket connection apears to have been closed.");
                    return;
                }
                Err(e) => log::error!("Unexpected error in websocket send: {:#?}", e),
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}

/// Converts the given md string to html
fn md_to_html(md: &str) -> String {
    let markdown_options = Options {
        parse: markdown::ParseOptions {
            constructs: markdown::Constructs {
                html_flow: true,
                html_text: true,
                definition: true,
                ..markdown::Constructs::gfm()
            },
            ..markdown::ParseOptions::gfm()
        },
        compile: markdown::CompileOptions {
            allow_dangerous_html: true,
            ..markdown::CompileOptions::gfm()
        },
    };

    post_process_html(to_html_with_options(md, &markdown_options).unwrap())
}

fn post_process_html(html: String) -> String {
    // Parse the HTML string into a DOM tree
    let document = kuchikiki::parse_html().one(html);

    // Find elements matching the selector
    let matching_elements = document
        .select("li>p>input[type=\"checkbox\"]")
        .expect("The selector is hard-coded.");

    for element in matching_elements {
        let checkbox = element.as_node();
        let li = checkbox
            .parent()
            .expect("The selector determines that these exist")
            .parent()
            .expect("The selector determines that these exist");
        let ul = li
            .parent()
            .expect("The selector determines that these exist");

        if let Some(checkbox_data) = checkbox.as_element() {
            let mut attributes = checkbox_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "task-list-item-checkbox".to_string());
        }

        if let Some(li_data) = li.as_element() {
            let mut attributes = li_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "task-list-item".to_string());
        }

        if let Some(ul_data) = ul.as_element() {
            let mut attributes = ul_data.attributes.borrow_mut();

            attributes.insert("class".to_string(), "contains-task-list".to_string());
        }
    }

    // Serialize the modified DOM back to HTML
    let mut output = Vec::new();
    document
        .serialize(&mut output)
        .expect("Serialization should never fail. All we did was add some classes.");
    String::from_utf8(output).expect("Converting to valid output should never fail.")
}

/// Returns the initial html, for when a client connects for the first time
fn initial_html(css: &str, body: &str) -> String {
    format!(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset=\"utf-8\"/>
        <title>My Project</title>
        <script src="./src/highlight.min.js"></script>
        <script src="./src/main.js" defer></script>
        <link id="md-stylesheet" rel="stylesheet" href="{}" />
    </head>
    <body class="markdown-body" id="body">
    {}
    </body>
    </html>
    "#,
        css, body
    )
}
