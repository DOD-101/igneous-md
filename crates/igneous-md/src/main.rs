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
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

mod cli;
mod client;
mod config;
mod convert;
mod errors;
mod export;
mod paths;
mod ws;

use cli::{Action, Cli};
use errors::Error;
use ws::upgrade_connection;

#[cfg(feature = "viewer")]
use {
    igneous_md_viewer::{Address, Viewer},
    std::thread,
};

#[cfg(feature = "generate_config")]
use std::{io, io::Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    SimpleLogger::new()
        .with_level(cli.log_level.to_level_filter())
        .init()
        .expect("Failed to init Logger.");

    let config = match config::Config::new(cli.config) {
        Ok(mut c) => {
            c.start_watching()
                .expect("Failed to start watching config dir");
            c
        }
        Err(e) => {
            log::error!("Failed to create `Config` Struct: {}", e);

            return Err(Box::new(e))?;
        }
    };

    // Convert the md file rather than launching the server, if the user passed the subcommand
    match cli.command {
        Action::Convert {
            path,
            css,
            export_name,
        } => {
            let html = convert::md_to_html(&fs::read_to_string(path).map_err(Error::InvalidInput)?);

            Ok(export::export(
                convert::initial_html(
                    &css.unwrap_or_else(|| {
                        config
                            .get_css_paths_clone()
                            .first()
                            .cloned()
                            .unwrap_or(PathBuf::new())
                    })
                    .to_string_lossy(),
                    &html,
                ),
                config.config_dir(),
                export_name,
            )
            .map_err(Error::ExportFailed)?)
        }
        #[cfg(feature = "generate_config")]
        Action::GenerateConfig { overwrite } => {
            if config.css_dir().exists() && !overwrite {
                return Err(Box::new(Error::ConfigDirExists) as Box<dyn std::error::Error>);
            }

            fs::create_dir_all(config.code_highlight_dir())
                .map_err(|e| Error::ConfigGenFailed(Box::new(e) as Box<dyn std::error::Error>))?;

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
                fs::create_dir_all(config.code_highlight_dir())
                    .map_err(|e| Error::ConfigGenFailed(Box::new(e)))?;

                // If compiled with generate_config, generate the config
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
                        config::generate::generate_config_files(&config.css_dir())
                            .await
                            .map_err(Error::ConfigGenFailed)?;
                    }
                }
            }

            if let Err(e) = fs::write("/tmp/ingeous-md", port.to_string()) {
                log::error!("Failed to write port to tmp file: {e}")
            };

            #[cfg(feature = "viewer")]
            if !no_viewer {
                let path = path.to_string_lossy().to_string();
                let css = css.map(|v| v.to_string_lossy().to_string());

                thread::spawn(move || {
                    let address = Address::new(
                        "localhost",
                        port,
                        update_rate,
                        css.as_deref(),
                        path.as_str(),
                    );
                    let client = Viewer::new(address);

                    client.start()
                });
            }

            let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;

            let config = Arc::new(RwLock::new(config));

            while let Ok((stream, other)) = listener.accept().await {
                log::info!("{}", other);
                tokio::spawn(upgrade_connection(stream, Arc::clone(&config)));
            }

            Ok(())
        }
    }
}
