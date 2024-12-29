//! Paths for the application
use home::home_dir;
use std::{path::PathBuf, sync::OnceLock};

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
    default_css: Option<PathBuf>,
}

impl Paths {
    pub fn new(css_dir: PathBuf, default_css: Option<PathBuf>) -> Self {
        Self {
            config_dir: default_config_path().into(),
            css_dir,
            default_css,
        }
    }

    pub fn get_css_dir(&self) -> PathBuf {
        self.css_dir.clone()
    }

    pub fn get_config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    pub fn get_default_css(&self) -> Option<PathBuf> {
        self.default_css.clone()
    }
}
