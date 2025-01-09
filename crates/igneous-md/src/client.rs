//! Module containing the [Client] struct.
//!
//! For more information see [Client]
use std::{io, path::PathBuf, sync::Arc, time::SystemTime};
use uuid::Uuid;

use crate::{config::Config, convert::md_to_html, paths::Paths};

/// All data relating to a client connected via websocket.
///
/// This Client is only dropped when the websocket is closed, which is most cases means the client
/// has disconnected.
///
/// This is where the live reloading of the `.md` files is implemented and all data needed for it
/// stored.
///
/// The Client also contains an [`Arc<Config>`] so that it can access the shared config state of the
/// application.
///
/// See also: [crate::handlers::upgrade_connection]
#[derive(Debug, Clone)]
pub struct Client {
    /// id of the client, currently not used
    #[allow(dead_code)]
    id: uuid::Uuid,
    /// Path to the`.md` on disk
    md_path: PathBuf,
    /// Last time the file was modified
    last_modified: SystemTime,
    /// The markdown from the file
    md: String,
    /// The markdown from the file
    html: String,
    /// [Config] shared between all clients
    pub config: Arc<Config>,
    /// The current position in [Config::css_paths]
    current_css_index: usize,
}

// NOTE: We could implement conversions to booleans here

/// Enum returned by [Client::changed] to indicate if a `.md` file has changed.
#[derive(Debug, Clone)]
pub enum MdChanged {
    /// The file has changed, contains the time of the latest change
    Changed(SystemTime),
    /// The file has not changed
    NotChanged,
}

impl Client {
    /// Attempt to create a new [Client]
    ///
    /// This can fail due to it containing a [Config].
    pub fn new(paths: &Paths, config: Arc<Config>) -> Self {
        Self {
            id: Uuid::new_v4(),
            md_path: paths.get_default_md(),
            md: String::new(),
            last_modified: SystemTime::UNIX_EPOCH,
            html: String::new(),
            config,
            current_css_index: 0,
        }
    }

    /// Read [Self::md_path] to a string and set [Self::md] to it
    fn update_md(&mut self) -> io::Result<()> {
        self.md = std::fs::read_to_string(&self.md_path)?;

        Ok(())
    }

    /// Check if [Self::md_path] has changed
    ///
    /// Checking is done via the files metadata.
    pub fn changed(&self) -> io::Result<MdChanged> {
        let last_modified = std::fs::metadata(&self.md_path)?.modified()?;

        if last_modified != self.last_modified {
            Ok(MdChanged::Changed(last_modified))
        } else {
            Ok(MdChanged::NotChanged)
        }
    }

    // NOTE: Being able to change this path without actually updating all the values derived from
    // it creates a strange state, where all of the data is false given the new path, but the user
    // must actually call a function to get data to update the data. This should probably be
    // addressed in the future.

    /// Set / Change [Self::md_path]
    pub fn set_md_path(&mut self, md_path: PathBuf) {
        self.md_path = md_path;
    }

    /// Getter function for [Self::md_path]
    #[allow(dead_code)]
    pub fn get_md_path(&self) -> PathBuf {
        self.md_path.clone()
    }

    /// Getter function for [Self::html]
    pub fn get_html(&self) -> String {
        self.html.clone()
    }

    /// [Self::get_latest_html_if_changed], but will always return html.
    #[allow(dead_code)]
    pub fn get_latest_html(&mut self) -> io::Result<String> {
        Ok(self
            .get_latest_html_if_changed()?
            .unwrap_or(self.html.clone()))
    }

    /// Get the next css file in [Self::config.css_paths], only returning [None] if it is empty.
    ///
    /// ## Note:
    ///
    /// The [Self::next_css] and [Self::previous_css] functions work by moving a pointer.
    ///
    /// Keep this in mind, as it differs from an [std::iter::Iterator]
    pub fn next_css(&mut self) -> Option<PathBuf> {
        if self.config.get_css_paths().is_empty() {
            return None;
        }

        self.current_css_index = (self.current_css_index + 1) % self.config.get_css_paths().len();

        self.config
            .get_css_paths()
            .get(self.current_css_index)
            .cloned()
    }

    /// Get the previous css file in [Self::config.css_paths], only returning [None] if it is empty.
    ///
    /// ## Note:
    ///
    /// See [Self::next_css]
    pub fn previous_css(&mut self) -> Option<PathBuf> {
        if self.config.get_css_paths().is_empty() {
            return None;
        }

        self.current_css_index = self
            .current_css_index
            .checked_sub(1)
            .unwrap_or(self.config.get_css_paths().len() - 1);

        self.config
            .get_css_paths()
            .get(self.current_css_index)
            .cloned()
    }

    /// Get the current css file from [Self::config.css_paths] without changing the index
    #[allow(dead_code)]
    pub fn current_css(&self) -> Option<PathBuf> {
        self.config
            .get_css_paths()
            .get(self.current_css_index)
            .cloned()
    }

    /// Checks if the`.md` file has changed, if so returning the new html else returning [None]
    pub fn get_latest_html_if_changed(&mut self) -> io::Result<Option<String>> {
        if let MdChanged::Changed(time) = self.changed()? {
            self.last_modified = time;
        } else {
            return Ok(None);
        }

        self.update_md()?;

        self.html = md_to_html(&self.md);

        Ok(Some(self.html.clone()))
    }
}
