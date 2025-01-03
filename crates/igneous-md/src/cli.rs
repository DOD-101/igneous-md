//! Module containing all CLI related functionality
use clap::{Args, Parser, Subcommand};
use rocket::log::LogLevel as RocketLogLevel;
use std::{fs, path::PathBuf, result::Result, str::FromStr};

use crate::{convert, export};

#[cfg(feature = "generate_config")]
use crate::config;
#[cfg(feature = "generate_config")]
use crate::paths::default_css_dir;
#[cfg(feature = "generate_config")]
use std::fmt::Display;

// HACK: I do not like the fact that args and command are not really mutually exclusive. I couldn't
// find a way to do this without using subcommands, which I do not want to do, since it doesn't
// make sense to force people to use a subcommand just to view the markdown, when that is the main
// purpose of the application.

/// Top Level Struct of the CLI
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version,
    about= "igneous-md | the simple and lightweight markdown viewer",
    long_about = None,
    subcommand_negates_reqs = true,
    // If we the fix the HACK then we could get rid of this 
    override_usage= "igneous-md [OPTIONS] <PATH|COMMAND>"
)]
pub struct Cli {
    #[command(subcommand)]
    /// Actions other than launching the server and viewer
    pub command: Option<Action>,
    #[command(flatten)]
    pub args: NormalArgs,
}

/// Args used when no [Cli::command] is passed
#[derive(Debug, Args)]
#[group(required = true, multiple = true)]
pub struct NormalArgs {
    /// Path to markdown file
    #[arg(value_name = "PATH", required = true)]
    pub path: Option<PathBuf>,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    pub css: Option<PathBuf>,
    /// Path to alternate css dir
    #[arg(long, value_name = "PATH")]
    pub css_dir: Option<PathBuf>,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    #[cfg(feature = "viewer")]
    pub no_viewer: bool,
    /// Log Level, aka. how verbose the application will be.
    #[arg(short, long, default_value = if cfg!(debug_assertions) {"INFO"} else {"WARN"})]
    pub log_level: UnifiedLevel,
    /// Port to run the server on
    #[arg(short, long, default_value = "2323")]
    pub port: u16,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    pub browser: bool,
}

impl Cli {
    /// Handle the different actions that need to happen if a [Cli::command] is passed
    pub fn handle_actions(&self) -> Result<ActionResult, Box<dyn std::error::Error>> {
        // Convert the md file rather than launching the server, if the user passed the subcommand
        if let Some(action) = &self.command {
            match action {
                Action::Convert { path, export_path } => {
                    let html = convert::md_to_html(
                        &fs::read_to_string(path).expect("Failed to read md file to string."),
                    );

                    if let Err(e) = export::export(
                        convert::initial_html(
                            &self.args.css.clone().unwrap_or_default().to_string_lossy(),
                            &html,
                        ),
                        export_path.clone().map(PathBuf::from),
                    ) {
                        log::error!("Failed to export md.");

                        return Err(Box::new(e));
                    }

                    Ok(ActionResult::Exit)
                }
                #[cfg(feature = "generate_config")]
                Action::GenerateConfig { overwrite } => {
                    if default_css_dir().exists() && !overwrite {
                        return Err(Box::new(ActionError::ConfigDirExists));
                    }

                    fs::create_dir_all(default_css_dir().join("hljs")).unwrap();

                    let rt = rocket::tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(config::generate::generate_config_files(default_css_dir()))?;

                    Ok(ActionResult::Exit)
                }
            }
        } else {
            Ok(ActionResult::Continue)
        }
    }
}

/// Actions other than launching the server to view markdown
#[derive(Debug, Subcommand)]
pub enum Action {
    /// Convert a md file to html and save it to disk
    Convert {
        /// The file to export
        path: PathBuf,
        /// The path of the output html
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

// NOTE: If this struct stays like this (just containing two options) it might be better to just
// have handle_actions return a bool

/// Possible return values for [Cli::handle_actions]
///
/// Used to tell the application if it should continue or exit
#[must_use = "ActionResult is used to tell the application what to do next."]
pub enum ActionResult {
    /// Exit the Application
    Exit,
    /// Continue as normal
    Continue,
}

/// Custom errors that may be emited by [Cli::handle_actions]
#[cfg(feature = "generate_config")]
#[derive(Debug)]
pub enum ActionError {
    ConfigDirExists,
}

#[cfg(feature = "generate_config")]
impl Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigDirExists => write!(
                f,
                "The config dir already exists. Run with -o to overwrite."
            ),
        }
    }
}

#[cfg(feature = "generate_config")]
impl std::error::Error for ActionError {}

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
