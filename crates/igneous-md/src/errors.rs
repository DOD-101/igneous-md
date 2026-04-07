//! Module containing custom errors emitted by igneous-md
//!
//! These mostly just wrap other errors, adding context.
use std::{
    error::Error as _,
    fmt::Debug,
    io,
    process::{ExitCode, Termination},
};

use thiserror::Error;

use crate::config::generate::urls;

/// Top-level errors that may occur when running different actions
///
/// Many of these are indented primarily to provide additional context to the user when returned
/// from main.
#[derive(Debug, Error)]
pub enum Error {
    /// IO encountered while generating the config
    ///
    /// More general than [Self::ConfigDirExists].
    #[error("Failed to generate the config {0}")]
    ConfigGenFailed(#[source] io::Error),
    /// Error when the config dir already exists
    #[error("The config dir already exists. Run with -o / --overwrite to continue regardless.")]
    ConfigDirExists,
    /// Headless client failed to launch
    #[error("The headless client failed required for conversion to launch.")]
    HeadlessClientLaunchFailed,
    /// The curl command was not found in path
    #[error(
        "`curl` was not found on your PATH.\n\
        Please either install curl or download the files manually:\n\
        {}\n\
        {}\n\
        {}\n\
        {}",
        urls::GH_DARK,
        urls::GH_DARK_HLJS,
        urls::GH_LIGHT,
        urls::GH_LIGHT_HLJS
    )]
    CurlNotFound,
    /// Curl was found but failed to launch
    #[error("An error occurred while launching curl.")]
    CurlLaunchFailed(#[source] io::Error),
    /// An error occurred while fetching the config files using curl
    #[error("Curl encountered an error while fetching the url `{0}`:\n{1}")]
    CurlFetch(String, String),
    /// The output of the curl wasn't a valid utf-8 string
    #[error("The output of the curl command for the url `{0} was invalid:\n{1}")]
    CurlOutputInvalid(String, std::string::FromUtf8Error),
    /// Failed to create the config
    #[error("Failed to create the config")]
    ConfigCreationFailed(#[source] io::Error),
    /// Failed to start watching the config dir
    #[error("Failed to watch the config dir")]
    WatchConfigDirFailed(#[source] notify::Error),
    /// Failed to launch the server
    #[error("Failed to launch the backend server")]
    ServerLaunchFailed(#[source] io::Error),
    /// Failed to register the ctrl_c signal
    #[error("Failed to register the ctrl_c signal to wait for exit")]
    SignalFailed(#[source] io::Error),
}

pub struct AppResult(pub Result<(), Error>);

impl Termination for AppResult {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {e}");
                let mut source = e.source();
                while let Some(cause) = source {
                    eprintln!("\nCaused by:\n    {cause}");
                    source = cause.source();
                }
                ExitCode::FAILURE
            }
        }
    }
}
