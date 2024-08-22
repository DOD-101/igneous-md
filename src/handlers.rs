//! Functions for handling incoming requests
//!
//! None of these functions should ever panic

use markdown::{to_html_with_options, Options};
use rouille::{
    match_assets, try_or_400,
    websocket::{self, SendError, Websocket},
    Request, Response,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::exit,
    thread,
    time::{Duration, SystemTime},
};

use crate::{css_path, Args};

/// Returns the css file name at the requested index
///
/// This function works by indexing the array of all css files and returning the appropriate one.
/// If the provided index in the url is too large it will use wrap the value around to a valid one.
///
/// The request must contain `?n={number}`
pub fn get_css_path(request: &Request, all_css: &[fs::DirEntry]) -> Response {
    let arguments =
        match serde_urlencoded::from_str::<HashMap<String, String>>(request.raw_query_string()) {
            Ok(args) if args.contains_key("n") => args.get("n").unwrap().to_owned(),
            _ => return Response::html("404 Error: Invalid URL-Parameters").with_status_code(404),
        };

    let index: usize = match arguments.parse::<usize>() {
        Ok(num) => num % all_css.len(),
        Err(_) => return Response::html("404 Error: Invalid URL-Parameters").with_status_code(404),
    };

    Response::text(all_css[index].path().file_name().unwrap().to_string_lossy())
}

/// Returns the requested css file if it exists
pub fn get_css(request: &Request) -> Response {
    let response = match_assets(request, css_path());

    if response.is_success() {
        return response;
    }

    println!("Failed to match css");
    Response::html("404 Error").with_status_code(404)
}

/// Returns the initial html converted from the md file
///
/// This function only gets called the first time a client requests a markdown document,
/// any subsequent updates are handled via the websocket see `upgrade_connection`.
pub fn get_inital_md(request: &Request, args: &Args, initial_css: &str) -> Response {
    let md = fs::read_to_string(format!(".{}", request.url()));

    if md.is_err() {
        return Response::html("404 Error").with_status_code(404);
    }

    let mut html = md_to_html(&md.unwrap());

    html = initial_html(initial_css, &html);

    if args.verbose {
        println!("SERVER: Sending: {html}");
    }

    Response::html(html)
}

/// Handles clients upgrading to websocket to receive file updates
///
/// This function will upgrade the connection to websocket and spawn a new thread for the
/// connection.
pub fn upgrade_connection(request: &Request, args: Args) -> Response {
    let (response, websocket) = try_or_400!(websocket::start(request, Some("md-data")));

    let file_path =
        PathBuf::from(&request.raw_query_string().split('=').collect::<Vec<_>>()[1].to_string());

    if !file_path.exists() || file_path.extension().unwrap_or_default() != "md" {
        println!("Something is wrong!");
        return Response::html("404 Error").with_status_code(404);
    }

    thread::spawn(move || {
        // Wait until the websocket is established
        let ws = websocket.recv().unwrap();

        ws_update_md(ws, &file_path, &args)
    });
    response
}

/// Internal logic for the websocket
///
/// Checks the metadata and every time it detects a file change will send the new markdown body to
/// the client.
fn ws_update_md(mut websocket: Websocket, file_path: &Path, args: &Args) {
    // TODO: In this entire function we are operating unter the assumption that the file exists,
    // because we checked that earlier, however it could still happen that we fail to read the
    // file, in which case this function will panic. This should be addressed in the future.

    let mut last_modified = SystemTime::UNIX_EPOCH;
    loop {
        let modified = fs::metadata(file_path).unwrap().modified().unwrap();

        if modified != last_modified {
            last_modified = modified;

            let html = md_to_html(&fs::read_to_string(file_path).unwrap());

            if args.verbose {
                println!("SERVER: Sending: {html}");
            }

            match websocket.send_text(&html) {
                Ok(_) => (),
                Err(SendError::Closed) => exit(0),
                Err(e) => println!("Unexpected error in websocket send: {:#?}", e),
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

    to_html_with_options(md, &markdown_options).unwrap()
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
