//! Module containing custom errors emitted by igneous-md
//!
//! These mostly just wrap other errors, adding context.
use std::fmt::{Debug, Display};

/// Custom errors that may occur when running different actions
pub enum Error {
    /// Error when generating the config fails
    ///
    /// More general than [Self::ConfigDirExists].
    ConfigGenFailed(Box<dyn std::error::Error>),
    /// Error when the config dir already exists
    #[cfg(feature = "generate_config")]
    ConfigDirExists,
    /// Error when exporting the md file to html fails
    ExportFailed(std::io::Error),
    /// Error when invalid input files are passed
    InvalidInput(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExportFailed(e) => write!(f, "Failed to export md. Underlying io error: {}", e),
            Self::InvalidInput(e) => write!(f, "Invalid input file. Underlying io error: {}", e),
            #[cfg(feature = "generate_config")]
            Self::ConfigDirExists => write!(
                f,
                "The config dir already exists. Run with -o to overwrite."
            ),
            #[cfg(feature = "generate_config")]
            Self::ConfigGenFailed(e) => {
                write!(f, "Failed to generate config. Underlying error: {}", e)
            }
            #[cfg(not(feature = "generate_config"))]
            Self::ConfigGenFailed(e) => {
                write!(f, "Failed to create config dir. Underlying error: {}", e)
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}
