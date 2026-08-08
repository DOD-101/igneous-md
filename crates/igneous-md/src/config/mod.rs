//! Module containing [Config] and other config-related items
//!
//! One [Config] struct is shared between all [crate::client::Client]s in the application.
//! Therefore, [Config] is solely responsible for holding config-related data that these clients
//! share and doesn't hold any state related to the config, such as, for example, what css files are
//! in use.
//!
//! The main item of this config is the [Config] struct, but it also contains [generate] to
//! generate the default config on disk.
use std::{io, path::PathBuf};

use crate::paths;
pub mod generate;

/// A CSS entry with its path and content
#[derive(Debug, Clone)]
pub struct CssEntry {
    /// Path to the CSS file (e.g., `/css/github-markdown-dark.css`)
    pub path: PathBuf,
    /// CSS content read from the file
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Where the config is located on disk
    pub config_dir: PathBuf,
    /// List of css entries within the [Config::css_dir]
    ///
    /// Each entry contains the path (starting with `/css/`) and the file content.
    pub css_entries: Vec<CssEntry>,
}

impl Config {
    /// Creates a new [`Config`] reading the [`Self::css_entries`] from disk
    pub fn new_from_disk(config_dir: PathBuf) -> io::Result<Self> {
        Ok(Self {
            css_entries: paths::read_css_dir(&paths::css_dir(&config_dir))?,
            config_dir,
        })
    }
}

#[cfg(test)]
impl Config {
    /// Creates a new Config for testing purposes
    ///
    /// `stylesheets` specifies the amount of css style sheets to create, in the format:
    ///
    /// `styleN.css`: where N is the number of the style sheet.
    pub fn new_testing(stylesheets: usize) -> Self {
        let mut css_entries = Vec::with_capacity(stylesheets);

        for n in 1..=stylesheets {
            css_entries.push(CssEntry {
                path: PathBuf::from(format!("/css/style{n}.css")),
                content: format!("/* style{n}.css */"),
            });
        }

        Self {
            config_dir: PathBuf::new(),
            css_entries,
        }
    }
}
