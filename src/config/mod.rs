//! Module containing [Config] and other config-related items
//!
//! The main item of this config is the [Config] struct, but it also contains [generate] to
//! generate the default config on disk.
use itertools::Itertools;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

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
    /// The current position in [Config::css_paths]
    current_css_index: usize,
}

impl Config {
    /// Attempt to create a new [Config]
    ///
    /// This may fail, since to set [Config::css_paths] we need to read from the Filesystem.
    pub fn new(paths: Paths) -> io::Result<Self> {
        let mut config = Self {
            config_dir: paths.get_config_dir(),
            css_dir: paths.get_css_dir(),
            css_paths: vec![],
            current_css_index: 0,
        };

        config.update_css_paths()?;

        Ok(config)
    }

    /// Get the next css file in [Self::css_paths], only returning [None] if it is empty.
    ///
    /// ## Note:
    ///
    /// The [Self::next_css] and [Self::previous_css] function work by moving a pointer.
    ///
    /// ```rust
    /// let config = Config {
    ///     config_dir: PathBuf::new(),
    ///     css_dir:  PathBuf::new();
    ///     css_paths: vec![1, 2, 3];
    ///     current_css_index: 0;
    /// }
    ///
    /// assert_eq!(config.next_css(), Some(1));
    /// assert_eq!(config.previous_css(), Some(3));
    ///
    /// ```
    /// Keep this in mind, as it differs from an [std::iter::Iterator]
    pub fn next_css(&mut self) -> Option<PathBuf> {
        if self.css_paths.is_empty() {
            return None;
        }

        self.current_css_index = (self.current_css_index + 1) % self.css_paths.len();

        self.css_paths.get(self.current_css_index).cloned()
    }

    /// Get the previous css file in [Self::css_paths], only returning [None] if it is empty.
    ///
    /// ## Note:
    ///
    /// See [Self::next_css]
    pub fn previous_css(&mut self) -> Option<PathBuf> {
        if self.css_paths.is_empty() {
            return None;
        }

        self.current_css_index = self
            .current_css_index
            .checked_sub(1)
            .unwrap_or(self.css_paths.len() - 1);

        self.css_paths.get(self.current_css_index).cloned()
    }

    /// Get the current css file from [Self::css_paths] without changing the index
    #[allow(dead_code)]
    pub fn current_css(&self) -> Option<PathBuf> {
        self.css_paths.get(self.current_css_index).cloned()
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
        let all_css: Vec<PathBuf> = read_css_dir(&self.css_dir)?;

        self.css_paths = all_css;

        log::info!("Updated css_paths: {:?}", self.css_paths);

        Ok(())
    }
}

// NOTE: Perhaps this should be under paths

/// Will attempt to read the given `css_dir` and organize the output
///
/// This function will:
///
/// 1. Only include `.css` files
///
/// 2. Return only names prefixed with `/css`
///
/// 3. Sort them by their name
pub fn read_css_dir(css_dir: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(fs::read_dir(css_dir)?
        .filter_map(|possible_entry| {
            let path = possible_entry.ok()?.path();

            if path.is_file() && path.extension().is_some_and(|s| s == "css") {
                return Some(
                    PathBuf::from("/css").join(
                        path.strip_prefix(css_dir)
                            .expect("We read the files from the css_dir."),
                    ),
                );
            }

            None
        })
        .sorted_by_key(|p| {
            PathBuf::from(
                p.file_name()
                    .expect("We checked that all entries are files."),
            )
        })
        .collect())
}
