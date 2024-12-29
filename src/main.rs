//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md --path path/to/file.md
//! ```
//! For more information see the usage docs.
//!

#[macro_use]
extern crate rocket;

use clap::Parser;
use rocket::{fs::FileServer, Build, Rocket};
use simple_logger::SimpleLogger;
use std::{path::PathBuf, process::exit, sync::Mutex, thread};

mod bidirectional_cycle;
mod client;
mod config;
mod convert;
mod handlers;
mod paths;

use handlers::*;
use paths::config_path;

#[launch]
fn rocket() -> Rocket<Build> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_level(args.log_level)
        .init()
        .unwrap();

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

    // The url of the md file, in the format:
    // localhost:port/path/to/file
    let md_url = format!("localhost:{}/?path={}", args.port, args.path);

    if args.browser && open::that_detached(&md_url).is_err() {
        log::warn!("Failed to open browser");
    }

    if !args.no_viewer {
        let client = client::Viewer::new(md_url);

        thread::spawn(move || client.start());
    }

    let css_dir = config_dir.join("css");

    rocket::build()
        .configure(rocket::Config {
            port: args.port,
            ..rocket::Config::default()
        })
        .manage(config)
        .manage(PathBuf::from(config_path()))
        .mount("/css", FileServer::from(css_dir).rank(1))
        .mount("/", FileServer::from("."))
        .mount(
            "/",
            routes![
                serve_main_js,
                serve_highlight_js,
                get_inital_md,
                upgrade_connection,
                save_html
            ],
        )
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    /// Path to markdown file
    path: String,
    /// Path to stylesheet within css dir
    // TODO: This doesn't work rn
    #[arg(short, long, value_name = "PATH")]
    css: Option<String>,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "Info")]
    log_level: log::LevelFilter,
    /// Port to run the server on
    #[arg(short, long, default_value = "2323")]
    port: u16,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    browser: bool,
}
