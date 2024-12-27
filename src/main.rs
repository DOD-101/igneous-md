//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md --path path/to/file.md
//! ```
//! For more information see the usage docs.
//!
use std::{path::PathBuf, sync::Mutex, thread};

use clap::Parser;
use rouille::{match_assets, router, start_server, Response};
use simple_logger::SimpleLogger;

mod bidirectional_cycle;
mod client;
mod config;
mod convert;
mod handlers;
mod paths;

use paths::config_path;

use std::process::exit;

fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();

    log::set_max_level(args.log_level);

    // address of the server
    let address = args.address.to_owned();

    let config_dir = PathBuf::from(config_path());

    // TODO: It might be nice for the user to be able to stop this
    if !config_dir.exists() && config::generate_config(&config_dir.join("css")).is_err() {
        log::error!("Failed to create default config.");
    }

    let config = Mutex::new(match config::Config::new(config_dir.clone()) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to create config: {:#?}", e);
            exit(1)
        }
    });

    let initial_css = config
        .lock()
        .unwrap()
        .next_css()
        .map(|path| path.to_string_lossy().to_string()) // Convert PathBuf to String
        .unwrap_or_else(String::new) // Default to an empty string if None
        .replace(config_dir.to_str().unwrap(), ""); // Replace config_dir if necessary

    // The url of the md file, in the format:
    // localhost:port/path/to/file
    let md_url = format!("{}/{}", address, args.path);

    log::info!("Starting live-reload server on {}", md_url);

    if args.browser && open::that_detached(&md_url).is_err() {
        log::warn!("Failed to open browser");
    }

    if !args.no_viewer {
        let client = client::Viewer::new(md_url);

        thread::spawn(move || client.start());
    }

    let hljs_src = include_str!("./highlight.min.js");
    let js_src = include_str!("./main.js");

    // TODO: Add a check here for if the path exists
    // IT would also be nice if this didn't need to be a mutex
    let default_path = Mutex::new(PathBuf::from(args.path));

    start_server(address, move |request| {
        log::info!("SERVER: Got request. With url: {:?}", request.url());
        router!(request,
            (GET) ["/src/main.js"] => {Response::from_data("text/javascript", js_src)},
            (GET) ["/src/highlight.min.js"] => {Response::from_data("text/javascript", hljs_src)},
            // WARN: Currently all Clients share the same config, which means that if one changes
            // the style sheet it will affect all others next style sheet as well. This needs to be
            // addressed before the next release.
            (GET) ["/api/get-css-path/next"] => {handlers::get_next_css_path(&mut config.lock().unwrap())},
            (GET) ["/api/get-css-path/prev"] => {handlers::get_prev_css_path(&mut config.lock().unwrap())},
            (GET) ["/css/{_path}", _path: String] => {handlers::get_css(request, config.lock().unwrap().get_css_dir().to_str().unwrap())},
            (GET) ["/css/hljs/{_path}", _path: String] => {handlers::get_css(request, config.lock().unwrap().get_css_dir().to_str().unwrap())},
            (POST) ["/api/post-html"] => {handlers::save_html(request)},
            (GET) ["/ws"] => {handlers::upgrade_connection(request, default_path.lock().unwrap().to_path_buf())},
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

                log::info!("Got invalid request: {:?}", request.url());
                Response::html("404 Error").with_status_code(404)
            }
        )
    });
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    /// Path to markdown file
    path: String,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    css: Option<String>,
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
