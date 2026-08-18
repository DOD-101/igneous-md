//! Module containing the [Client] struct.
//!
//! For more information see [Client]
use crate::{handshake::WsQueryParams, paths};
use futures_util::{SinkExt as _, StreamExt as _, stream::SplitSink};
use igneous_md_protocol::{AsMsg as _, ClientMsg, ServerMsg};
use kuchikiki::traits::*;
use std::{
    io,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{net::TcpStream, sync::mpsc, time};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message as WsMessage};

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
    /// Id for identifying the client
    id: u16,
    /// Path to the`.md` on disk
    md_path: PathBuf,
    /// First value [`Self::md_path`] was set to
    ///
    /// Needed to allow for [`igneous_md_protocol::ClientMsg::RedirectDefault`]
    initial_md_path: PathBuf,
    /// Last time the file was modified
    last_modified: SystemTime,
    /// The markdown from the file
    md: String,
    /// The html `<main>` element of the file
    html: String,
    /// [Config] shared between all clients
    pub config: Arc<Config>,
    /// The current position in [Config::css_entries]
    ///
    /// If this is [None] then there are no css entries available.
    current_css_index: Option<u16>,

    input_rx: mpsc::Receiver<ClientInputMsg>,
}

pub enum ClientInputMsg {
    ConfigUpdate(Arc<Config>),
}

#[derive(Debug, Clone)]
pub struct ClientHandle {
    pub input_tx: mpsc::Sender<ClientInputMsg>,
}

impl ClientHandle {
    fn new(input_tx: mpsc::Sender<ClientInputMsg>) -> Self {
        Self { input_tx }
    }
}

/// Enum returned by [Client::changed] to indicate if a `.md` file has changed.
#[derive(Debug, Clone)]
pub enum MdChanged {
    /// The file has changed, contains the time of the latest change
    Changed(SystemTime),
    /// The file has not changed
    NotChanged,
}

impl Client {
    /// Create a new [Client]
    ///
    /// Additionally returns a sender to send messages to the client.
    pub fn new(id: u16, md_path: PathBuf, config: Arc<Config>) -> (Self, ClientHandle) {
        let current_css_index = if config.css_entries.is_empty() {
            None
        } else {
            Some(0)
        };

        let (input_tx, input_rx) = mpsc::channel(8);

        (
            Self {
                id,
                initial_md_path: md_path.clone(),
                md_path,
                md: String::new(),
                last_modified: SystemTime::UNIX_EPOCH,
                html: String::new(),
                config,
                current_css_index,
                input_rx,
            },
            ClientHandle::new(input_tx),
        )
    }

    /// Start the client
    ///
    /// Returns the clients id on finish
    pub async fn start(
        mut self,
        ws_stream: WebSocketStream<TcpStream>,
        params: WsQueryParams,
    ) -> u16 {
        let mut interval =
            time::interval(Duration::from_millis(params.update_rate.unwrap_or(1000)));

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // TODO: This should ideally be cleaned up (using a custom Stream type?). There are 4 different locations a message can be
        // sent from which can lead to inconsistencies in logging
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Ok(Some(html)) = self.get_latest_html_if_changed() {
                        let msg = ServerMsg::HtmlUpdate { html };
                        log::info!("Sending ws message: {}", msg.name());

                        let _ = ws_write.send(msg.as_msg()).await;
                    }
                }

                Some(input_msg) = self.input_rx.recv() => self.handle_input_msg(input_msg, &mut ws_write).await,

                incoming = ws_read.next() => {
                    match incoming {
                        Some(Ok(message)) => {
                            match message {
                                WsMessage::Text(msg_string) => {
                                    if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg_string) {
                                        log::info!("Received ws message: {}", client_msg.name());
                                        log::debug!("Full received ws message: {:?}", client_msg);

                                        let return_msg = self.handle_client_msg(client_msg);

                                        if let Ok(()) = ws_write.send(return_msg.as_msg()).await {
                                            log::info!("Sent ws response: {}", return_msg.name());
                                            log::debug!("Full sent ws message: {:?}", return_msg);
                                        } else {
                                            log::error!("Failed to send server response.")
                                        }
                                    } else {
                                        log::warn!("Invalid client message: {}", msg_string)
                                    }
                                },
                                WsMessage::Close(_) => {
                                    log::info!("Client initiated connection close");
                                    break;
                                }
                                msg => {
                                    log::warn!("Received unknown ws message: {msg:?}")
                                }
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("Error receiving message: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }

        self.id
    }

    /// [upgrade_connection()] uses this to handle the incoming messages from the client
    fn handle_client_msg(&mut self, msg: ClientMsg) -> ServerMsg {
        match msg {
            ClientMsg::ChangeCss { index, relative } => {
                self.change_current_css_index(index, relative);

                // If there are no css entries, still respond with an (empty) update so that clients
                // waiting on a css update don't hang.
                ServerMsg::CssUpdate {
                    css: self.current_css().unwrap_or_default(),
                }
            }
            ClientMsg::RequestExport => ServerMsg::Export {
                path: paths::export_path(&self.config.config_dir),
            },
            ClientMsg::Redirect { path } => {
                self.set_md_path(path);

                match self.get_latest_html() {
                    Ok(html) => ServerMsg::HtmlUpdate { html },
                    Err(e) => ServerMsg::Error { msg: e.to_string() },
                }
            }
            ClientMsg::RedirectDefault => {
                self.reset_md_path_to_initial();

                match self.get_latest_html() {
                    Ok(html) => ServerMsg::HtmlUpdate { html },
                    Err(e) => ServerMsg::Error { msg: e.to_string() },
                }
            }
            ClientMsg::CheckServer => ServerMsg::Success,
        }
    }

    async fn handle_input_msg(
        &mut self,
        msg: ClientInputMsg,
        ws_write: &mut SplitSink<WebSocketStream<TcpStream>, WsMessage>,
    ) {
        match msg {
            ClientInputMsg::ConfigUpdate(config) => {
                self.config = config;
                if let Some(css) = self.current_css()
                    && let Err(e) = ws_write.send(ServerMsg::CssUpdate { css }.as_msg()).await
                {
                    log::error!("Failed to send server response: {e}")
                };
            }
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

    /// [Self::get_latest_html_if_changed], but will always return html.
    pub fn get_latest_html(&mut self) -> io::Result<String> {
        Ok(self
            .get_latest_html_if_changed()?
            .unwrap_or(self.html.clone()))
    }

    /// Get the current css content from [Self::config.css_entries] without changing the index
    pub fn current_css(&self) -> Option<String> {
        self.current_css_index.and_then(|i| {
            self.config
                .css_entries
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

            let max_index = self.config.css_entries.len() as i16 - 1;

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
                    .is_some_and(|v| (v as usize) < self.config.css_entries.len()),
                "current_css_index is invalid: max-index: {:?}; index: {:?}",
                self.config.css_entries.len() - 1,
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

            let current_css_index = if config_stylesheets == 0 {
                None
            } else {
                Some(0)
            };

            let (_, input_rx) = mpsc::channel(8);

            Self {
                id: 0,
                initial_md_path: PathBuf::new(),
                md_path: PathBuf::new(),
                md: String::new(),
                last_modified: SystemTime::UNIX_EPOCH,
                html: String::new(),
                config: Arc::new(config),
                current_css_index,
                input_rx,
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
