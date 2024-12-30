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

use clap::{Parser, Subcommand};
use rocket::{config::LogLevel as RocketLogLevel, fs::FileServer, Build, Rocket};
use simple_logger::SimpleLogger;
use std::{fs, path::PathBuf, process::exit, str::FromStr, thread};

mod client;
mod config;
mod convert;
mod export;
mod handlers;
mod paths;

use handlers::*;
use paths::{default_config_path, Paths};

#[launch]
fn rocket() -> Rocket<Build> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_level(args.log_level.into())
        .init()
        .expect("Failed to init Logger.");

    // Convert the md file rather than launching the server, if the user passed the subcommand
    if let Some(Action::Convert { export_path }) = args.command {
        let html = convert::md_to_html(
            &fs::read_to_string(args.path.clone()).expect("Failed to read md file to string."),
        );
        if export::export(
            convert::initial_html(&args.css.unwrap_or(PathBuf::new()).to_string_lossy(), &html),
            export_path.map(PathBuf::from),
        )
        .is_err()
        {
            log::error!("Failed to export md.");
            exit(1);
        }

        exit(0);
    }

    #[cfg(feature = "generate_config")]
    if !default_config_path().exists()
        && config::generate_config(&default_config_path().join("css")).is_err()
    {
        log::error!("Failed to create default config.");
    }

    let paths = match Paths::new(
        args.css_dir.unwrap_or(default_config_path().join("css")),
        args.css.map(|p| PathBuf::from("/css").join(p)),
    ) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to create Paths: {}", e);

            #[cfg(not(feature = "generate_config"))]
            log::info!("Check that the config dir exists and contains css files.");
            log::info!("igneous-md has been compiled without the generate_config feature.");

            exit(1)
        }
    };

    // The url of the md file, in the format:
    // localhost:port/?path=path/to/file
    let md_url = format!(
        "localhost:{}/?path={}",
        args.port,
        args.path.to_string_lossy()
    );

    if args.browser && open::that_detached(&md_url).is_err() {
        log::warn!("Failed to open browser");
    }

    if !args.no_viewer {
        let client = client::Viewer::new(md_url);

        thread::spawn(move || client.start());
    }

    let css_dir = paths.get_css_dir();

    if !css_dir.exists() {
        log::error!("Css dir: {} doesn't exist. Exiting.", css_dir.display());
        exit(1);
    }

    rocket::build()
        .configure(rocket::Config {
            port: args.port,
            log_level: args.log_level.into(),
            ..rocket::Config::default()
        })
        .manage(paths)
        .mount("/css", FileServer::from(css_dir).rank(1))
        .mount("/", FileServer::from("."))
        .mount(
            "/",
            routes![
                serve_main_js,
                serve_highlight_js,
                get_initial_md,
                upgrade_connection,
            ],
        )
}

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
struct Args {
    #[command(subcommand)]
    /// Actions other than launching the server and viewer
    command: Option<Action>,
    /// Path to markdown file
    path: PathBuf,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    css: Option<PathBuf>,
    /// Path to alternate css dir
    #[arg(long, value_name = "PATH")]
    css_dir: Option<PathBuf>,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "Info")]
    log_level: UnifiedLevel,
    /// Port to run the server on
    #[arg(short, long, default_value = "2323")]
    port: u16,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    browser: bool,
}

/// Actions other than launching the server to view markdown
#[derive(Debug, Subcommand)]
enum Action {
    /// Convert a md file to html and save it to disk
    Convert {
        #[arg(short, long, value_name = "PATH")]
        export_path: Option<PathBuf>,
    },
}

/// Wrapper around [log::LevelFilter] to allow conversion to [RocketLogLevel]
#[derive(Clone, Debug, Copy)]
struct UnifiedLevel(log::LevelFilter);

impl From<RocketLogLevel> for UnifiedLevel {
    fn from(value: RocketLogLevel) -> Self {
        match value {
            RocketLogLevel::Off => Self(log::LevelFilter::Off),
            RocketLogLevel::Critical => Self(log::LevelFilter::Error),
            RocketLogLevel::Normal => Self(log::LevelFilter::Info),
            RocketLogLevel::Debug => Self(log::LevelFilter::Debug),
        }
    }
}

impl From<UnifiedLevel> for RocketLogLevel {
    fn from(value: UnifiedLevel) -> Self {
        match value {
            UnifiedLevel(log::LevelFilter::Off) => Self::Off,
            UnifiedLevel(log::LevelFilter::Error) => Self::Critical,
            UnifiedLevel(log::LevelFilter::Warn) | UnifiedLevel(log::LevelFilter::Info) => {
                Self::Normal
            }
            UnifiedLevel(log::LevelFilter::Debug) | UnifiedLevel(log::LevelFilter::Trace) => {
                Self::Debug
            }
        }
    }
}

impl From<UnifiedLevel> for log::LevelFilter {
    fn from(value: UnifiedLevel) -> Self {
        value.0
    }
}

impl FromStr for UnifiedLevel {
    type Err = log::ParseLevelError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UnifiedLevel(log::LevelFilter::from_str(s)?))
    }
}
