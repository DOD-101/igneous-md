//! Paths for the application
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
    config_dir: PathBuf,
    css_dir: PathBuf,
    default_css: PathBuf,
}

impl Paths {
    pub fn new(css_dir: PathBuf, default_css: Option<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            config_dir: default_config_path().into(),
            default_css: default_css.unwrap_or(Self::determine_default_css(&css_dir)?),
            css_dir,
        })
    }

    pub fn get_css_dir(&self) -> PathBuf {
        self.css_dir.clone()
    }

    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    pub fn get_default_css(&self) -> PathBuf {
        self.default_css.clone()
    }

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
