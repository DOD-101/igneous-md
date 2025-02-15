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
use rocket::fs::FileServer;
use simple_logger::SimpleLogger;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod cli;
mod client;
mod config;
mod convert;
mod errors;
mod export;
mod handlers;
mod paths;

use cli::{Action, Cli};
use errors::Error;
use handlers::*;
use paths::default_css_dir;
use paths::Paths;

#[cfg(feature = "viewer")]
use {igneous_md_viewer::Viewer, std::thread};

#[cfg(feature = "generate_config")]
use std::{io, io::Write};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    SimpleLogger::new()
        .with_level(cli.log_level.into())
        .init()
        .expect("Failed to init Logger.");

    // Convert the md file rather than launching the server, if the user passed the subcommand
    match cli.command {
        Action::Convert {
            path,
            export_path,
            css,
        } => {
            let html = convert::md_to_html(
                &fs::read_to_string(path).expect("Failed to read md file to string."),
            );

            Ok(export::export(
                convert::initial_html(&css.clone().unwrap_or_default().to_string_lossy(), &html),
                export_path.clone().map(PathBuf::from),
            )
            .map_err(Error::ExportFailed)?)
        }
        #[cfg(feature = "generate_config")]
        Action::GenerateConfig { overwrite } => {
            if default_css_dir().exists() && !overwrite {
                return Err(Box::new(Error::ConfigDirExists) as Box<dyn std::error::Error>);
            }

            fs::create_dir_all(default_css_dir().join("hljs"))
                .map_err(|e| Error::ConfigGenFailed(Box::new(e) as Box<dyn std::error::Error>))?;

            config::generate::generate_config_files(default_css_dir()).await?;

            Ok(())
        }
        Action::View {
            path,
            css,
            css_dir,
            port,
            browser,
            #[cfg(feature = "viewer")]
            no_viewer,
        } => {
            // TODO: In the future it might be nice to check if the dir contains no css, rather than just
            // checking if it exists. However as it stands currently users can avoid the prompt, by
            // creating the dirs.

            // Check if the config exists
            if !default_css_dir().exists() {
                // Always at least create the dir
                fs::create_dir_all(default_css_dir().join("hljs"))
                    .map_err(|e| Error::ConfigGenFailed(Box::new(e)))?;

                // If compiled with generate_config generate the config
                #[cfg(feature = "generate_config")]
                {
                    print!("No config found. Would you like to generate the default config? [(y)es/(N)o]: ");

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
                        config::generate::generate_config_files(default_css_dir())
                            .await
                            .map_err(Error::ConfigGenFailed)?;
                    }
                }
            }

            let paths = match Paths::new(
                path,
                css_dir.unwrap_or(default_css_dir().to_path_buf()),
                css.map(|p| PathBuf::from("/css").join(p)),
            ) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to create `Paths` Struct: {}", e);

                    return Err(Box::new(e));
                }
            };

            let config = match config::Config::new(&paths) {
                Ok(mut c) => {
                    c.start_watching()
                        .expect("Failed to start watching config dir");
                    Arc::new(Mutex::new(c))
                }
                Err(e) => {
                    log::error!("Failed to create `Config` Struct: {}", e);

                    return Err(Box::new(e));
                }
            };

            // The url of the md file, in the format:
            // localhost:port
            let address = format!("localhost:{}", port);

            if browser && open::that_detached(&address).is_err() {
                log::warn!("Failed to open browser");
            }

            #[cfg(feature = "viewer")]
            if !no_viewer {
                let client = Viewer::new(address);

                thread::spawn(move || client.start());
            }

            rocket::build()
                .configure(rocket::Config {
                    port,
                    log_level: cli.log_level.into(),
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
                .register("/", catchers![not_found, internal_error])
                .launch()
                .await?;

            Ok(())
        }
    }
}
