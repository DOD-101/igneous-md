//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md --path path/to/file.md
//! ```
//! For more information see the usage docs.
//!
use clap::Parser;
use gtk::{prelude::*, Window, WindowType};
use rouille::{match_assets, router, start_server, Response};
use std::{fs, process::exit, thread};
use webkit2gtk::{WebView, WebViewExt};

mod handlers;
mod paths;

use paths::{config_path, default_css_path};

use simple_logger::SimpleLogger;
fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();

    log::set_max_level(args.log_level);

    // address of the server
    let address = args.address.to_owned();

    let css_dir = &args.css_dir;

    let mut all_css: Vec<fs::DirEntry> = match fs::read_dir(css_dir) {
        Ok(dir) => dir
            .filter_map(|css| match css {
                Ok(entry) => {
                    if entry.path().is_dir() {
                        return None;
                    }

                    log::info!("CSS Option: {:#?}", entry);

                    Some(entry)
                }
                Err(error) => {
                    log::warn!("Could not read entry in css folder: {:#?}", error);
                    None
                }
            })
            .collect(),
        Err(_) => {
            log::error!("Failed to read css dir: {}", css_dir);
            exit(1)
        }
    };

    all_css.sort_by_key(|a| a.file_name());

    let initial_css = match args.css.to_owned() {
        Some(css) => css,
        None => all_css[0].file_name().to_string_lossy().to_string(),
    };

    // localhost:port/path/to/file
    let md_url = format!("{}/{}", address, args.path);

    log::info!("Starting live-reload server on {}", md_url);

    if args.browser && open::that_detached(&md_url).is_err() {
        log::warn!("Failed to open browser");
    }

    if !args.no_viewer {
        thread::spawn(move || client(&md_url));
    }

    start_server(address, move |request| {
        log::info!("SERVER: Got request. With url: {:?}", request.url());
        router!(request,
            (GET) (/api/get-css-path) => {handlers::get_css_path(request, &all_css)},
            (POST) (/api/post-html) => {handlers::save_html(request)},
            (GET) (/ws) => {handlers::upgrade_connection(request)},
            _ => {
                if request.url().ends_with(".md") {
                        return handlers::get_inital_md(request, &initial_css);
                }

                {
                    // Match any assets in the current dir
                    let response = match_assets(request, ".");

                    if response.is_success() {
                        return response;
                    }
                }

                // if that fails check if it is a .css, in which case it is probably
                // in the css dir
                // TODO: This creates an issue where we might accidentally get a different
                // stylesheet than the user wants, if they have one with the same name in
                // the current working dir
                if request.url().ends_with(".css") {
                    return handlers::get_css(request, &args.css_dir);
                }

                log::info!("Got invalid request: {:?}", request.url());
                Response::html("404 Error").with_status_code(404)
            }
        )
    });
}

/// Starts the markdown viewer
fn client(addr: &str) {
    log::info!("Starting client on {addr}");
    if gtk::init().is_err() {
        log::error!("Failed to init gtk. Needed for viewer.");
        exit(1)
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("igneous-md viewer");
    window.set_default_size(800, 600);

    let view = WebView::new();
    view.load_uri(&format!("http://{addr}"));

    window.add(&view);

    window.show_all();

    gtk::main()
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    /// Path to markdown file
    #[arg(short, long)]
    path: String,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    css: Option<String>,
    /// Path to the css dir
    ///
    /// Defaults to config_dir/css, if that doesn't exist uses the example.
    #[arg(long, value_name = "PATH", default_value = &**default_css_path())]
    css_dir: String,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "Info")]
    log_level: log::LevelFilter,
    #[arg(short, long, default_value = "localhost:2323")]
    address: String,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    browser: bool,
}
