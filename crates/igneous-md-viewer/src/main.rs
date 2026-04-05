//! The viewer component of igneous_md
//!
//! This binary just launches the viewer.
//!
//! It's useful for when you accidentally closed the viewer, but don't want to restart the whole
//! server.
use std::{error::Error, fs};

use clap::{CommandFactory, Parser};
use igneous_md_viewer::{Address, Viewer};

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            Cli::command().get_name(),
            &mut std::io::stdout(),
        );
    }

    let addr = Address::new(
        "localhost",
        cli.port
            .map_or_else(read_port, Ok)
            .expect("Failed to get port of server."),
        cli.update_rate,
        cli.css.as_deref(),
        cli.path.as_str(),
    );

    if cli.browser {
        if open::that_detached(addr.to_string()).is_err() {
            println!("WARN: Failed to open browser");
        }

        return;
    }

    let viewer = Viewer::new(addr, false);

    viewer.start();
}

fn read_port() -> Result<u16, Box<dyn Error>> {
    Ok(fs::read_to_string("/tmp/ingeous-md")?
        .parse()
        .expect("Invalid port file in tmp."))
}

#[derive(Debug, Parser)]
/// Igneous-md viewer
///
/// Used to connect to an already running igneous-md server.
pub struct Cli {
    /// Path of the md file to view
    pub path: String,
    /// Port of the server
    ///
    /// If none is supplied the viewer will attempt to read `/tmp/igneous-md` where the server
    /// writes it's port to on start.
    #[arg(short, long)]
    pub port: Option<u16>,
    /// Path to the initial css to use
    #[arg(short, long)]
    pub css: Option<String>,
    /// Generate shell completions
    #[arg(long)]
    pub completions: Option<clap_complete::Shell>,
    /// How often to check for updates (in ms)
    #[arg(short, long, default_value = "1000")]
    pub update_rate: u64,
    /// Open in browser instead of standalone
    #[arg(long, visible_aliases = ["web"])]
    pub browser: bool,
}
