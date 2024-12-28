//! Functions for handling incoming requests
//!
//! None of these functions should ever panic
use kuchikiki::traits::*;
use markdown::{to_html_with_options, Options};
use rocket::{
    futures::{SinkExt, StreamExt},
    http::Status,
    response::content::*,
    tokio::{
        select,
        time::{self, Duration},
    },
    State,
};
use rocket_ws::{Channel, Message, WebSocket};
use std::{fs, path::PathBuf, sync::Mutex};

use crate::{client::Client, config::Config, config_path};

#[get("/api/get-css-path/next")]
/// Gets the previous style-sheet from the given Config
pub fn get_next_css_path(config: &State<Mutex<Config>>) -> Option<String> {
    let mut config = config.lock().unwrap();
    config.next_css().map(|p| p.to_string_lossy().to_string())
}

#[get("/api/get-css-path/prev")]
/// Gets the previous style-sheet from the given Config
pub fn get_prev_css_path(config: &State<Mutex<Config>>) -> Option<String> {
    let mut config = config.lock().unwrap();
    config
        .previous_css()
        .map(|p| p.to_string_lossy().to_string())
}

#[get("/src/main.js")]
pub fn serve_main_js() -> RawJavaScript<&'static str> {
    RawJavaScript(include_str!("./main.js"))
}

#[get("/src/highlight.min.js")]
pub fn serve_highlight_js() -> RawJavaScript<&'static str> {
    RawJavaScript(include_str!("./highlight.min.js"))
}

/// Returns the initial html converted from the md file
///
/// This function only gets called the first time a client requests a markdown document,
/// any subsequent updates are handled via the websocket see `upgrade_connection`.
///
//  TODO: This function is just bad. The path makes it conflict with everything else.
#[get("/<file..>", rank = 2)]
pub fn get_inital_md(file: PathBuf, config: &State<Mutex<Config>>) -> Option<RawHtml<String>> {
    let mut html = match fs::read_to_string(file.clone()) {
        Ok(md) => md_to_html(&md),
        Err(e) => {
            log::error!("Failed to read .md file {}", file.to_string_lossy());
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

/// Handles clients upgrading to websocket to receive file updates
///
/// This function will upgrade the connection to websocket and spawn a new thread for the
/// connection.
#[get("/ws/<path..>")]
pub async fn upgrade_connection(ws: WebSocket, path: PathBuf) -> Channel<'static> {
    let mut client = Client::new(PathBuf::from(".").join(path));

    // Set up the WebSocket channel using Rocket's channel method
    ws.channel(move |mut stream| Box::pin(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {

            // Use tokio::select! to handle both incoming messages and broadcast messages
            select! {
                _ = interval.tick()=> {
                    if let Ok(Some(html)) = client.get_latest_html_if_changed() {
                        log::info!("Sending new html");
                        let _ = stream.send(Message::Text(html)).await;
                    }
                }

                // Handle incoming messages from the client
                incoming = stream.next() => {
                    match incoming {
                        Some(Ok(message)) => {
                            match message {
                                Message::Text(client_message) => {
                                    println!("Received message from client: {}", client_message);
                                    // Here you could add your own logic to process client messages
                                }
                                Message::Close(_) => {
                                    println!("Client initiated connection close");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => {
                            println!("Error receiving message: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }

Ok(())
}))
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
