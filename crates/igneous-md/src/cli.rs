//! Module containing all CLI related functionality
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::paths::DEFAULT_CONFIG_DIR;

/// Top Level Struct of the CLI
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version,
    about= "igneous-md | the simple and lightweight markdown viewer",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    /// What to do
    pub command: Action,
    /// Log Level, aka. how verbose the application will be.
    #[arg(short, long, default_value = if cfg!(debug_assertions) {"INFO"} else {"WARN"})]
    pub log_level: log::Level,
    /// Change path to the config
    #[arg(long, default_value = DEFAULT_CONFIG_DIR.as_os_str(), value_name = "PATH")]
    pub config: PathBuf,
}

/// Actions other than launching the server to view markdown
#[derive(Debug, Subcommand)]
pub enum Action {
    /// View a markdown file
    #[command(visible_alias = "v")]
    View {
        /// Path to markdown file
        #[arg(value_name = "PATH", required = true)]
        path: PathBuf,
        /// Path to stylesheet within css dir
        #[arg(short, long, value_name = "PATH")]
        css: Option<PathBuf>,
        /// Start server without viewer
        #[arg(long, default_value = "false")]
        #[cfg(feature = "viewer")]
        no_viewer: bool,
        /// Port to run the server on
        #[arg(short, long, default_value = "0")]
        port: u16,
        // TODO: Add this option back in here once viewer is updated to not need an http server
        // /// Open browser tab
        // #[arg(short, long, visible_aliases = ["web"], default_value = "false")]
        // browser: bool,
        /// How often to check for updates (in ms)
        #[arg(short, long, default_value = "1000")]
        update_rate: u64,
    },
    /// Convert a md file to html and save it to disk
    ///
    /// The file will be saved to the specified config dir.
    #[command(visible_alias = "export")]
    Convert {
        /// The file to convert
        path: PathBuf,
        /// Path to set the css stylesheet to
        #[arg(short, long, value_name = "PATH")]
        css: Option<PathBuf>,
        /// Path to save the file to
        ///
        /// Defaults to saving it into the config dir.
        #[arg(short, long, value_name = "FILE")]
        export_path: Option<PathBuf>,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Generate the default config. Requires an internet connection
    GenerateConfig {
        /// Whether to overwrite the contents of the config dir
        #[arg(short, long)]
        overwrite: bool,
    },
}
