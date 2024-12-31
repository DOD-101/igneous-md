use clap::{Parser, Subcommand};
use rocket::log::LogLevel as RocketLogLevel;
use std::{fmt::Display, fs, path::PathBuf, result::Result, str::FromStr};

use crate::{config, convert, export, paths::default_css_dir};

/// Struct containing all command line options
/// For more information see [clap documentation](https://docs.rs/clap/latest/clap/index.html)
#[derive(Parser, Debug)]
#[command(version, about= "igneous-md | the simple and lightweight markdown viewer", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    /// Actions other than launching the server and viewer
    pub command: Option<Action>,
    /// Path to markdown file
    pub path: PathBuf,
    /// Path to stylesheet within css dir
    #[arg(short, long, value_name = "PATH")]
    pub css: Option<PathBuf>,
    /// Path to alternate css dir
    #[arg(long, value_name = "PATH")]
    pub css_dir: Option<PathBuf>,
    /// Start server without viewer
    #[arg(long, default_value = "false")]
    pub no_viewer: bool,
    /// Will only print when starting server and on serious errors
    #[arg(short, long, default_value = "Info")]
    pub log_level: UnifiedLevel,
    /// Port to run the server on
    #[arg(short, long, default_value = "2323")]
    pub port: u16,
    /// Open browser tab
    #[arg(short, long, default_value = "false")]
    pub browser: bool,
}

impl Args {
    pub fn handle_actions(&self) -> Result<ActionResult, Box<dyn std::error::Error>> {
        // Convert the md file rather than launching the server, if the user passed the subcommand
        if let Some(action) = &self.command {
            match action {
                Action::Convert { export_path } => {
                    let html = convert::md_to_html(
                        &fs::read_to_string(self.path.clone())
                            .expect("Failed to read md file to string."),
                    );

                    if let Err(e) = export::export(
                        convert::initial_html(
                            &self.css.clone().unwrap_or_default().to_string_lossy(),
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
        #[arg(short, long, value_name = "PATH")]
        export_path: Option<PathBuf>,
    },

    /// Generate the default config. Requires an internet connection
    #[cfg(feature = "generate_config")]
    GenerateConfig {
        #[arg(short, long)]
        overwrite: bool,
    },
}

/// Possible return values for [Args::handle_actions]
#[must_use = "ActionResult is used to tell the application what to do next."]
pub enum ActionResult {
    Exit,
    Continue,
}

/// Custom errors that may be emited by [Args::handle_actions]
#[derive(Debug)]
pub enum ActionError {
    ConfigDirExists,
}

impl Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigDirExists => write!(f, "The config dir already exists."),
        }
    }
}

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
