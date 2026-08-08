//! igneous-md | the simple and lightweight markdown viewer
//!
//! # Usage
//!
//! ```
//! igneous-md view path/to/file.md
//!
//! igneous-md convert path/to/file.md
//! ```
//! For more information see the usage docs.
//!

use clap::{CommandFactory, Parser};
use simple_logger::SimpleLogger;
use std::fs;

mod cli;
mod client;
mod config;
mod convert;
mod errors;
mod paths;
mod server;
mod ws;

use cli::{Action, Cli};
use errors::Error;

use crate::{config::Config, errors::AppResult, paths::attempt_delete_port_file};

#[cfg(feature = "viewer")]
use {
    igneous_md_viewer::{Address, Viewer},
    std::thread,
};

use std::{
    io::{self, Write},
    time::Duration,
};
use tokio::{net::TcpListener, task::AbortHandle, time::sleep};

#[tokio::main]
async fn main() -> AppResult {
    AppResult(run().await)
}

async fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    SimpleLogger::new()
        .with_level(cli.log_level.to_level_filter())
        .init()
        .expect("Failed to init Logger.");

    match cli.command {
        #[cfg(feature = "viewer")]
        Action::Convert {
            path,
            css,
            export_path,
        } => {
            let default_export_path = paths::export_path(&cli.config);

            let port = get_free_port().await.unwrap();

            let config = Config::new_from_disk(cli.config.clone())
                .map_err(Error::ConfigFromDiskFailed)?
                .into();
            tokio::spawn(async move {
                server::Server::new(config).start(port).await;
            });

            let path = path.to_string_lossy().to_string();
            let css = css.map(|v| v.to_string_lossy().to_string());

            let export_path = export_path.map_or(default_export_path, |p| {
                if !p.is_absolute() {
                    return std::env::current_dir().expect("Failed to get cwd!").join(p);
                }

                p
            });

            thread::spawn(move || {
                let export_str: String =
                    form_urlencoded::byte_serialize(export_path.to_string_lossy().as_bytes())
                        .collect();
                let address = Address::new(
                    "localhost",
                    port,
                    1000,
                    css.as_deref(),
                    path.as_str(),
                    Some(&export_str),
                );
                let client = Viewer::new(address, true);

                client.start()
            });

            // TODO: get rid of this fragile timing (also see main.js)

            // wait for printing to complete
            sleep(Duration::from_secs(1)).await;

            // TODO: add check if printing succeeded

            Ok(())
        }
        Action::GenerateConfig { overwrite } => {
            if paths::css_dir(&cli.config).exists() && !overwrite {
                return Err(Error::ConfigDirExists);
            }

            fs::create_dir_all(paths::code_highlight_dir(&cli.config))
                .map_err(Error::ConfigGenFailed)?;

            config::generate::generate_config_files(&paths::css_dir(&cli.config)).await?;

            Ok(())
        }
        Action::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                Cli::command().get_name(),
                &mut std::io::stdout(),
            );

            Ok(())
        }
        Action::View {
            path,
            css,
            port,
            update_rate,
            #[cfg(feature = "viewer")]
            no_viewer,
        } => {
            // TODO: In the future it might be nice to check if the dir contains no css, rather than just
            // checking if it exists. However as it stands currently users can avoid the prompt, by
            // creating the dirs.

            // Check if the config exists
            if !paths::css_dir(&cli.config).exists() {
                // Always at least create the dir
                fs::create_dir_all(paths::code_highlight_dir(&cli.config))
                    .map_err(Error::ConfigGenFailed)?;

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
                    // TODO: this is confusing, we generate config files, but place them in the css dir
                    config::generate::generate_config_files(&paths::css_dir(&cli.config)).await?;
                }
            }

            let mut existing_port = None;
            // if no port was given explicitly
            if port == 0 {
                match fs::read_to_string(paths::SERVER_PORT_FILE) {
                    Ok(content) => {
                        if let Ok(port) = content.parse::<u16>() {
                            if server::test_server_connection(port).await {
                                log::info!("Connecting to existing server on port {port}");
                                existing_port = Some(port);
                            }
                        } else {
                            log::debug!(
                                "{} is invalid. Attempting to delete.",
                                paths::SERVER_PORT_FILE
                            );
                            paths::attempt_delete_port_file();
                        }
                    }
                    Err(e) => {
                        log::warn!("Could not read {}: {e}", paths::SERVER_PORT_FILE);
                    }
                }
            }

            let mut server_handle: Option<AbortHandle> = None;
            let tcp_port = if let Some(p) = existing_port {
                p
            } else {
                let p = get_free_port().await.unwrap();

                let config = Config::new_from_disk(cli.config.clone())
                    .map_err(Error::ConfigFromDiskFailed)?
                    .into();
                server_handle = Some(
                    tokio::spawn(async move {
                        server::Server::new(config).start(p).await;
                    })
                    .abort_handle(),
                );

                p
            };

            #[cfg(feature = "viewer")]
            let viewer_handle = if !no_viewer {
                let path = path.to_string_lossy().to_string();
                let css = css.map(|v| v.to_string_lossy().to_string());

                // TODO: If in the future we can change this to a Command it would (a) simplify the
                // build process somewhat since the server would no longer rely on the viewer (b)
                // allow the process to exit if we only need to launch the viewer if another server
                // is already running

                Some(thread::spawn(move || {
                    let address = Address::new(
                        "localhost",
                        tcp_port,
                        update_rate,
                        css.as_deref(),
                        path.as_str(),
                        None,
                    );
                    let client = Viewer::new(address, false);

                    client.start()
                }))
            } else {
                None
            };

            // exit if we didn't start the server
            let Some(handle) = server_handle else {
                // wait on the viewer if it was started (see todo above)
                #[cfg(feature = "viewer")]
                {
                    if let Some(vh) = viewer_handle {
                        vh.join().unwrap();
                    }
                }
                return Ok(());
            };

            tokio::signal::ctrl_c().await.map_err(Error::SignalFailed)?;

            handle.abort();

            attempt_delete_port_file();

            Ok(())
        }
    }
}

/// Get a free tcp port
///
/// Since the port is not bound when returned, users should be quick to use this value otherwise
/// risking a TOCTOU error.
async fn get_free_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    Ok(port)
    // listener is dropped here, freeing the port
}
