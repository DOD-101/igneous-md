//! Module containing all CLI related functionality
use clap::{Parser, Subcommand};
use rocket::log::LogLevel as RocketLogLevel;
use std::{path::PathBuf, result::Result, str::FromStr};

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
    pub log_level: UnifiedLevel,
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
        /// Path to alternate css dir
        #[arg(long, value_name = "PATH")]
        css_dir: Option<PathBuf>,
        /// Start server without viewer
        #[arg(long, default_value = "false")]
        #[cfg(feature = "viewer")]
        no_viewer: bool,
        /// Port to run the server on
        #[arg(short, long, default_value = "2323")]
        port: u16,
        /// Open browser tab
        #[arg(short, long, default_value = "false")]
        browser: bool,
    },
    /// Convert a md file to html and save it to disk
    Convert {
        /// The file to convert
        path: PathBuf,
        /// Path to set the css stylesheet to
        #[arg(short, long, value_name = "PATH")]
        css: Option<PathBuf>,
        /// Path to save the html to
        #[arg(short, long, value_name = "PATH")]
        export_path: Option<PathBuf>,
    },

    /// Generate the default config. Requires an internet connection
    #[cfg(feature = "generate_config")]
    GenerateConfig {
        /// Whether to overwrite the contents of the config dir
        #[arg(short, long)]
        overwrite: bool,
    },
}

/// Wrapper around [log::LevelFilter] to allow conversion to [RocketLogLevel]
#[derive(Clone, Debug, Copy)]
pub struct UnifiedLevel(log::LevelFilter);

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
