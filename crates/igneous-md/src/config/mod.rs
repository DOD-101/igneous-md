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

pub mod generate;

/// A CSS entry with its path and content
#[derive(Debug, Clone)]
pub struct CssEntry {
    /// Path to the CSS file (e.g., `/css/github-markdown-dark.css`)
    pub path: PathBuf,
    /// CSS content read from the file
    pub content: String,
}

/// Struct containing all information relating to the config, including the css files.
#[derive(Debug)]
pub struct Config {
    /// Where the config is located on disk
    config_dir: PathBuf,
    /// List of css entries within the [Config::css_dir]
    ///
    /// Each entry contains the path (starting with `/css/`) and the file content.
    css_entries: Arc<Mutex<Vec<CssEntry>>>,
    /// Sender for [notify::Event]s
    pub update_sender: tokio::sync::broadcast::Sender<notify::Event>,
    /// The watcher, if it is running
    watcher: Option<notify::RecommendedWatcher>,
}

impl Config {
    /// Attempt to create a new [Config]
    ///
    /// This may fail, since to set [Config::css_entries] we need to read from the Filesystem.
    pub fn new(config_dir: PathBuf) -> io::Result<Self> {
        Ok(Self {
            css_entries: Arc::new(Mutex::new(crate::paths::read_css_dir(
                &config_dir.join("css/"),
            )?)),
            config_dir,
            update_sender: broadcast::channel(1).0,
            watcher: None,
        })
    }

    /// Get [Self::css_entries]
    pub fn get_css_entries_clone(&self) -> Vec<CssEntry> {
        self.css_entries.lock().unwrap().clone()
    }

    /// How many css entries there are
    pub fn css_entries_len(&self) -> usize {
        self.css_entries.lock().unwrap().len()
    }

    /// Directory where the css files are located
    pub fn css_dir(&self) -> PathBuf {
        self.config_dir.join("css")
    }

    /// Directory where the css files for code highlighting are located
    pub fn code_highlight_dir(&self) -> PathBuf {
        self.config_dir.join("css/hljs")
    }

    /// Get [field@Self::config_dir]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Start watching the [Self::config_dir]
    ///
    /// After this [Self::update_sender] will start sending events.
    pub fn start_watching(&mut self) -> notify::Result<()> {
        let config_dir = self.config_dir.clone();
        let css_dir = self.css_dir();
        let css_entries = Arc::clone(&self.css_entries);

        let sender = self.update_sender.clone();

        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    if !event.kind.is_access() {
                        log::info!("Config update");
                        let _ = sender.send(event);
                        *css_entries.lock().unwrap() =
                            crate::paths::read_css_dir(&css_dir).unwrap();
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
            css_entries: Arc::new(Mutex::new(css_entries)),
            update_sender: broadcast::channel(1).0,
            watcher: None,
        }
    }
}
