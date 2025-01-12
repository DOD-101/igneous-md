//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md path/to/file.md
//!
//! igneous-md convert path/to/file.md
//! ```
//! For more information see the usage docs.
//!

#[macro_use]
extern crate rocket;

use clap::Parser;
use rocket::{fs::FileServer, Build, Rocket};
use simple_logger::SimpleLogger;
use std::{
    fs,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

mod cli;
mod client;
mod config;
mod convert;
mod export;
mod handlers;
mod paths;

use cli::{ActionResult, Cli};
use handlers::*;
use paths::default_css_dir;
use paths::Paths;

#[cfg(feature = "viewer")]
use igneous_md_viewer::Viewer;
#[cfg(feature = "viewer")]
use std::thread;

#[cfg(feature = "generate_config")]
use std::{io, io::Write};

#[launch]
fn rocket() -> Rocket<Build> {
    let cli = Cli::parse();

    SimpleLogger::new()
        .with_level(cli.args.log_level.into())
        .init()
        .expect("Failed to init Logger.");

    match cli.handle_actions() {
        Ok(ActionResult::Continue) => (),
        Ok(ActionResult::Exit) => exit(0),
        Err(e) => {
            log::error!("{}", e);

            exit(1)
        }
    }

    // TODO: In the future it might be nice to check if the dir contains no css, rather than just
    // checking if it exists. However as it stands currently users can avoid the prompt, by
    // creating the dirs.

    // Check if the config exists
    if !default_css_dir().exists() {
        // Always at least create the dir
        if let Err(e) = fs::create_dir_all(default_css_dir().join("hljs")) {
            log::error!(
                "Failed to create css_dir: {} With error: {}",
                default_css_dir().to_string_lossy(),
                e
            );

            exit(1)
        }

        // If compiled with generate_config generate the config
        #[cfg(feature = "generate_config")]
        {
            print!(
                "No config found. Would you like to generate the default config? [(y)es/(N)o]: "
            );

            io::stdout().flush().expect("Failed to flush stdout.");

            let mut user_input = String::new();

            io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read input.");

            if user_input
                .to_lowercase()
                .chars()
                .next()
                .is_some_and(|c| c == 'y')
            {
                let rt = tokio::runtime::Runtime::new().unwrap();

                if let Err(e) =
                    rt.block_on(config::generate::generate_config_files(default_css_dir()))
                {
                    log::error!("Failed to create default config: {}", e);

                    exit(1)
                }
            }
        }
    }

    let paths = match Paths::new(
        cli.args.path.expect("Path is required."),
        cli.args.css_dir.unwrap_or(default_css_dir().to_path_buf()),
        cli.args.css.map(|p| PathBuf::from("/css").join(p)),
    ) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to create Paths: {}", e);

            exit(1)
        }
    };

    let config = match config::Config::new(&paths) {
        Ok(mut c) => {
            c.start_watching()
                .expect("Failed to start watching config dir");
            Arc::new(Mutex::new(c))
        }
        Err(e) => {
            log::error!("Failed to create Config: {}", e);

            exit(1)
        }
    };

    // The url of the md file, in the format:
    // localhost:port
    let address = format!("localhost:{}", cli.args.port);

    if cli.args.browser && open::that_detached(&address).is_err() {
        log::warn!("Failed to open browser");
    }

    #[cfg(feature = "viewer")]
    if !cli.args.no_viewer {
        let client = Viewer::new(address);

        thread::spawn(move || client.start());
    }

    rocket::build()
        .configure(rocket::Config {
            port: cli.args.port,
            log_level: cli.args.log_level.into(),
            ..rocket::Config::default()
        })
        .manage(paths)
        .manage(config)
        .mount("/", FileServer::from("."))
        .mount(
            "/",
            routes![
                serve_main_js,
                serve_highlight_js,
                serve_css,
                get_initial_md,
                upgrade_connection,
            ],
        )
}
