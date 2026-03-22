//! Module containing the [Client] struct.
//!
//! For more information see [Client]
use kuchikiki::traits::*;
use std::{
    io,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::SystemTime,
};
use tokio::sync::broadcast;

use crate::{config::Config, convert::md_to_html};

/// Struct representing a client connection to the server
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
/// See also: [crate::ws::upgrade_connection()]
#[derive(Debug)]
pub struct Client {
    /// Path to the`.md` on disk
    md_path: PathBuf,
    /// First value [`Self::md_path`] was set to
    ///
    /// Needed to allow for [`crate::ws::msg::ClientMsg::RedirectDefault`]
    initial_md_path: PathBuf,
    /// Last time the file was modified
    last_modified: SystemTime,
    /// The markdown from the file
    md: String,
    /// The html `<main>` element of the file
    html: String,
    /// [Config] shared between all clients
    pub config: Arc<RwLock<Config>>,
    /// Receiver of [notify::Event]s
    pub config_update_receiver: broadcast::Receiver<notify::Event>,
    /// The current position in [Config::css_entries]
    ///
    /// If this is [None] then there are no css entries available.
    current_css_index: Option<u16>,
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
    pub fn new(md_path: PathBuf, config: Arc<RwLock<Config>>) -> Self {
        let (config_update_receiver, current_css_index);
        {
            let config = config.read().unwrap();

            config_update_receiver = config.update_sender.subscribe();
            current_css_index = if config.css_entries_len() > 0 {
                Some(0)
            } else {
                None
            }
        }

        Self {
            initial_md_path: md_path.clone(),
            md_path,
            md: String::new(),
            last_modified: SystemTime::UNIX_EPOCH,
            html: String::new(),
            config,
            config_update_receiver,
            current_css_index,
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

    /// Set [Self::md_path]
    pub fn set_md_path(&mut self, md_path: PathBuf) {
        self.md_path = md_path;
    }

    /// Set [Self::md_path] back to [Self::initial_md_path]
    pub fn reset_md_path_to_initial(&mut self) {
        self.md_path = self.initial_md_path.clone();
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

    /// Get the current css content from [Self::config.css_entries] without changing the index
    pub fn current_css(&self) -> Option<String> {
        self.current_css_index.and_then(|i| {
            self.config
                .read()
                .expect("Failed to lock config. This should never happen.")
                .get_css_entries_clone()
                .get(i as usize)
                .map(|entry| entry.content.clone())
        })
    }

    /// Checks if the`.md` file has changed, if so returning the new html else returning [None]
    pub fn get_latest_html_if_changed(&mut self) -> io::Result<Option<String>> {
        if let MdChanged::Changed(time) = self.changed()? {
            self.last_modified = time;
        } else {
            return Ok(None);
        }

        self.update_md()?;

        let html = md_to_html(&self.md);

        let document = kuchikiki::parse_html().one(html);

        let mut body = Vec::new();
        document
            .select_first("main")
            .expect("Html must have a main")
            .as_node()
            .serialize(&mut body)
            .expect("Serialization should never fail, if it does there is a bug.");

        self.html =
            String::from_utf8(body).expect("Converting main element to string should never fail.");

        Ok(Some(self.html.clone()))
    }

    /// Change the current css
    ///
    /// Makes sure the value is always valid
    ///
    /// If relative is `false` ignores the current value.
    pub fn change_current_css_index(&mut self, change: i16, relative: bool) {
        if let Some(i) = self.current_css_index {
            let raw_index = if relative { i as i16 + change } else { change };

            let max_index = self.config.read().unwrap().css_entries_len() as i16 - 1;

            let index = if max_index == 0 {
                // since it is the only option
                0
            } else if raw_index < 0 {
                // + because the number is negative
                (max_index + 1) + (raw_index % max_index)
            } else if raw_index > max_index {
                raw_index % (max_index + 1)
            } else {
                raw_index
            };

            self.current_css_index = Some(index as u16);

            debug_assert!(
                self.current_css_index
                    .is_some_and(|v| (v as usize) < self.config.read().unwrap().css_entries_len()),
                "current_css_index is invalid: max-index: {:?}; index: {:?}",
                self.config.read().unwrap().css_entries_len() - 1,
                self.current_css_index
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Client {
        pub fn new_testing(config_stylesheets: usize) -> Self {
            let config = Config::new_testing(config_stylesheets);

            let (config_update_receiver, current_css_index);
            {
                config_update_receiver = config.update_sender.subscribe();
                current_css_index = if config.css_entries_len() > 0 {
                    Some(0)
                } else {
                    None
                }
            }

            Self {
                initial_md_path: PathBuf::new(),
                md_path: PathBuf::new(),
                md: String::new(),
                last_modified: SystemTime::UNIX_EPOCH,
                html: String::new(),
                config_update_receiver,
                config: Arc::new(RwLock::new(config)),
                current_css_index,
            }
        }
    }

    #[test]
    fn next_css() {
        let mut client = Client::new_testing(3);

        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));
        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));
        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));
        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));
        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));
    }

    #[test]
    fn previous_css() {
        let mut client = Client::new_testing(3);

        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));
        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));
        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));
        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));
        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));
    }

    #[test]
    fn next_previous_mixed_1() {
        let mut client = Client::new_testing(3);

        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));

        client.change_current_css_index(2, true);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));

        client.change_current_css_index(-2, true);
        assert_eq!(client.current_css(), Some("/* style3.css */".to_string()));

        client.change_current_css_index(0, false);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));

        client.change_current_css_index(9, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));

        client.change_current_css_index(10, false);
        assert_eq!(client.current_css(), Some("/* style2.css */".to_string()));
    }

    #[test]
    fn next_previous_on_single() {
        let mut client = Client::new_testing(1);

        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));

        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));

        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));

        client.change_current_css_index(2, false);
        assert_eq!(client.current_css(), Some("/* style1.css */".to_string()));
    }

    #[test]
    fn next_previous_on_empty() {
        let mut client = Client::new_testing(0);

        client.change_current_css_index(-1, true);
        assert_eq!(client.current_css(), None);

        client.change_current_css_index(1, true);
        assert_eq!(client.current_css(), None);

        client.change_current_css_index(1, false);
        assert_eq!(client.current_css(), None);

        client.change_current_css_index(-1, false);
        assert_eq!(client.current_css(), None);
    }
}
