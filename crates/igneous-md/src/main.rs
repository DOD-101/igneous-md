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

use crate::errors::AppResult;

#[cfg(feature = "viewer")]
use {
    igneous_md_viewer::{Address, Viewer},
    std::thread,
};

use std::{
    io::{self, Write},
    time::Duration,
};
use tokio::time::sleep;

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

    let config = match config::Config::new(cli.config) {
        Ok(mut c) => {
            c.start_watching().map_err(Error::WatchConfigDirFailed)?;

            c
        }
        Err(e) => {
            return Err(Error::ConfigCreationFailed(e));
        }
    };

    match cli.command {
        #[cfg(feature = "viewer")]
        Action::Convert {
            path,
            css,
            export_path,
        } => {
            let default_export_path = config.export_path();
            let handle = server::launch_server(0, config)
                .await
                .map_err(Error::ServerLaunchFailed)?;

            let tcp_port = handle.port();

            let path = path.to_string_lossy().to_string();
            let css = css.map(|v| v.to_string_lossy().to_string());

            thread::spawn(move || {
                let address =
                    Address::new("localhost", tcp_port, 1000, css.as_deref(), path.as_str());
                let client = Viewer::new(address, true);

                client.start()
            });

            // TODO: When we add proper ClientHandles in server.rs (see TODO there) we can also
            // improve the code here to no longer rely on these timings

            // wait for client to start
            sleep(Duration::from_millis(1000)).await;

            let mut launch_tries = 0;
            loop {
                if let Some(tx) = handle.get_client_sender(0) {
                    tx.send(ws::msg::ServerMsg::Export {
                        path: export_path
                            .map(|p| {
                                if !p.is_absolute() {
                                    return std::env::current_dir()
                                        .expect("Failed to get cwd!")
                                        .join(p);
                                }

                                p
                            })
                            .unwrap_or(default_export_path),
                    })
                    .map_err(|_| Error::HeadlessClientLaunchFailed)?;

                    break;
                }
                launch_tries += 1;

                if launch_tries > 5 {
                    log::error!(
                        "Failed to start headless client! Cannot convert markdown to pdf. "
                    );
                    return Err(Error::HeadlessClientLaunchFailed);
                }

                log::warn!("Headless client hasn't started. Waiting further.");

                sleep(Duration::from_millis(1000)).await;
            }

            // wait for printing to complete
            sleep(Duration::from_millis(1000)).await;

            Ok(())
        }
        Action::GenerateConfig { overwrite } => {
            if config.css_dir().exists() && !overwrite {
                return Err(Error::ConfigDirExists);
            }

            fs::create_dir_all(config.code_highlight_dir()).map_err(Error::ConfigGenFailed)?;

            config::generate::generate_config_files(&config.css_dir()).await?;

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
            if !config.css_dir().exists() {
                // Always at least create the dir
                fs::create_dir_all(config.code_highlight_dir()).map_err(Error::ConfigGenFailed)?;

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
                    config::generate::generate_config_files(&config.css_dir()).await?;
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

            let mut handle = None;
            let tcp_port = if let Some(p) = existing_port {
                p
            } else {
                let h = server::launch_server(port, config)
                    .await
                    .map_err(Error::ServerLaunchFailed)?;

                let p = h.port();

                handle = Some(h);

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
                    );
                    let client = Viewer::new(address, false);

                    client.start()
                }))
            } else {
                None
            };

            // exit if we didn't start the server
            let Some(handle) = handle else {
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

            handle.stop().expect("Failed to stop server properly.");

            Ok(())
        }
    }
}
