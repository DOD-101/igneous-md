//! Module containing the [Client] struct.
//!
//! For more information see [Client]
use std::{io, path::PathBuf, time::SystemTime};
use uuid::Uuid;

use crate::{config::Config, convert::md_to_html, paths::Paths};

/// Struct containing all data needed about the client by the websocket
///
/// This is where the live reloading of the `.md` files is implemented and all data needed for it
/// stored.
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
    /// Contained [Config] for the client
    pub config: Config,
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
    pub fn new(md_path: PathBuf, paths: Paths) -> io::Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            md_path,
            md: String::new(),
            last_modified: SystemTime::UNIX_EPOCH,
            html: String::new(),
            config: Config::new(paths)?,
        })
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
