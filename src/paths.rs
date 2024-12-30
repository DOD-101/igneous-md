//! External paths for the application
//!
//! The [Paths] struct is the only state managed by the [rocket] server, since it is needed to init
//! new [crate::client::Client]s.
use home::home_dir;
use std::{
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::config::read_css_dir;

/// Returns the config path for the application
pub fn default_config_path() -> &'static PathBuf {
    static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
    CONFIG_PATH.get_or_init(|| {
        home_dir()
            .expect("Couldn't find the home dir!")
            .join(".config/igneous-md/")
    })
}

/// Paths used by the application
#[derive(Clone, Debug)]
pub struct Paths {
    /// The dir containing the config for the application. See [default_config_path()]
    ///
    /// Currently this isn't very important, since we primarily care about the [Self::css_dir]
    config_dir: PathBuf,
    /// The dir containing the css files
    css_dir: PathBuf,
    /// The first css file every client receives.
    ///
    /// This is not an actual path on disk, but rather the API path for the css file
    default_css: PathBuf,
}

impl Paths {
    /// Attempt to create a new [Paths]
    ///
    /// This can fail, only if no `default_css` is supplied, since it needs to read css files from
    /// disk.
    pub fn new(css_dir: PathBuf, default_css: Option<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            config_dir: default_config_path().into(),
            default_css: default_css.unwrap_or(Self::determine_default_css(&css_dir)?),
            css_dir,
        })
    }

    /// Getter function for [Self::config_dir]
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    /// Getter function for [Self::css_dir]
    pub fn get_css_dir(&self) -> PathBuf {
        self.css_dir.clone()
    }

    /// Getter function for [Self::default_css]
    pub fn get_default_css(&self) -> PathBuf {
        self.default_css.clone()
    }

    /// Used by [Self::new] to set [Self::default_css] by reading `css_dir`
    ///
    /// If there are no css files it will return an empty [PathBuf]
    fn determine_default_css(css_dir: &Path) -> io::Result<PathBuf> {
        let all_css = read_css_dir(css_dir)?;

        let default_css = all_css.first().map_or(PathBuf::new(), |p| p.to_path_buf());

        log::info!(
            "Automatically set default_css to: {}",
            default_css.to_string_lossy()
        );

        Ok(default_css)
    }
}
