//! Module containing [Config] and other config-related items
//!
//! One [Config] struct is shared between all [crate::client::Client]s in the application.
//! Therefore, [Config] is solely responsible for holding config-related data that these clients
//! share and doesn't hold any state related to the config, such as, for example, what css files are
//! in use.
//!
//! The main item of this config is the [Config] struct, but it also contains [generate] to
//! generate the default config on disk.
use notify::Watcher;
use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use crate::paths::Paths;

#[cfg(feature = "generate_config")]
pub mod generate;

/// Struct containing all information relating to the config, including the css files.
#[derive(Debug)]
pub struct Config {
    /// Path to the config
    config_dir: PathBuf,
    /// Path to the dir, where the css files are
    css_dir: PathBuf,
    /// List of css files within the [Config::css_dir]
    ///
    /// Paths all start with `/css/` followed by the name of the file.
    css_paths: Arc<Mutex<Vec<PathBuf>>>,
    /// Sender for [notify::Event]s
    pub update_sender: tokio::sync::broadcast::Sender<notify::Event>,
    /// The watcher, if it is running
    watcher: Option<notify::RecommendedWatcher>,
}

impl Config {
    /// Attempt to create a new [Config]
    ///
    /// This may fail, since to set [Config::css_paths] we need to read from the Filesystem.
    pub fn new(paths: &Paths) -> io::Result<Self> {
        Ok(Self {
            config_dir: paths.get_config_dir(),
            css_dir: paths.get_css_dir(),
            css_paths: Arc::new(Mutex::new(crate::paths::read_css_dir(
                &paths.get_css_dir(),
            )?)),
            update_sender: broadcast::channel(1).0,
            watcher: None,
        })
    }

    /// Get [Self::css_paths]
    pub fn get_css_paths_clone(&self) -> Vec<PathBuf> {
        self.css_paths.lock().unwrap().clone()
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

    /// Start watching the [Self::config_dir]
    ///
    /// After this [Self::update_sender] will start sending events.
    pub fn start_watching(&mut self) -> notify::Result<()> {
        let config_dir = self.config_dir.clone();
        let css_dir = self.css_dir.clone();
        let css_paths = self.css_paths.clone();

        let sender = self.update_sender.clone();

        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    if !event.kind.is_access() {
                        log::info!("Config update");
                        let _ = sender.send(event);
                        *css_paths.lock().unwrap() = crate::paths::read_css_dir(&css_dir).unwrap();
                    }
                }
            })?;

        log::info!("Watching config dir: {}", config_dir.to_string_lossy());

        watcher
            .watch(&config_dir, notify::RecursiveMode::Recursive)
            .unwrap();

        self.watcher = Some(watcher);

        Ok(())
    }
}

#[cfg(test)]
impl Config {
    pub fn new_testing(css_paths: Vec<PathBuf>) -> Self {
        Self {
            config_dir: PathBuf::new(),
            css_dir: PathBuf::new(),
            css_paths: Arc::new(Mutex::new(css_paths)),
            update_sender: broadcast::channel(1).0,
            watcher: None,
        }
    }
}
