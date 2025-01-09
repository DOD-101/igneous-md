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

use crate::paths::Paths;

#[cfg(feature = "generate_config")]
pub mod generate;

/// Struct containing all information relating to the config, including the css files.
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the config
    config_dir: PathBuf,
    /// Path to the dir, where the css files are
    css_dir: PathBuf,
    /// List of css files within the [Config::css_dir]
    ///
    /// Paths all start with `/css/` followed by the name of the file.
    css_paths: Vec<PathBuf>,
}

impl Config {
    /// Attempt to create a new [Config]
    ///
    /// This may fail, since to set [Config::css_paths] we need to read from the Filesystem.
    pub fn new(paths: &Paths) -> io::Result<Self> {
        let mut config = Self {
            config_dir: paths.get_config_dir(),
            css_dir: paths.get_css_dir(),
            css_paths: vec![],
        };

        config.update_css_paths()?;

        Ok(config)
    }

    /// Get [Self::css_paths]
    pub fn get_css_paths(&self) -> &Vec<PathBuf> {
        &self.css_paths
    }

    /// Get [Self::css_dir]
    ///
    /// This will just be [Paths::get_css_dir] of the [Paths] passed to [Self::new]
    #[allow(dead_code)]
    pub fn get_css_dir(&self) -> &PathBuf {
        &self.css_dir
    }

    /// Get [Self::config_dir]
    ///
    /// This will just be [Paths::get_config_dir] of the [Paths] passed to [Self::new]
    #[allow(dead_code)]
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Reads [Self::css_dir], updating [Self::css_paths]
    pub fn update_css_paths(&mut self) -> io::Result<()> {
        let all_css: Vec<PathBuf> = crate::paths::read_css_dir(&self.css_dir)?;

        self.css_paths = all_css;

        log::info!("Updated css_paths: {:?}", self.css_paths);

        Ok(())
    }
}
