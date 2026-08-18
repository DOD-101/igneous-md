//! Shared WebSocket protocol types for igneous-md.
//!
//! This crate defines the message types exchanged between the server and any
//! viewer/client over the WebSocket connection. Both the server and any
//! frontend import these types to stay in sync.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{EnumIs, IntoStaticStr};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

pub trait AsMsg {
    /// Convert [Self] into a [WsMessage]
    fn as_msg(&self) -> WsMessage;
}

/// Possible messages sent by the server
#[derive(Serialize, Deserialize, Debug, IntoStaticStr, PartialEq, Eq, EnumIs)]
#[serde(tag = "t", content = "c")]
pub enum ServerMsg {
    /// Updated CSS for the html content
    CssUpdate {
        /// Css content
        css: String,
    },
    /// Updated HTML rendered from markdown
    HtmlUpdate {
        /// Html content
        html: String,
    },
    /// Request the client export the current html to the specified path
    Export {
        /// The path to export to
        path: PathBuf,
    },
    /// Server is shutting down
    Exit {
        /// If the exit is due to an error
        error: bool,
    },
    /// Arbitrary success message
    Success,
    /// Arbitrary error message
    Error {
        /// Message describing in human-readable format the issue
        msg: String,
    },
}

impl AsMsg for ServerMsg {
    fn as_msg(&self) -> WsMessage {
        WsMessage::Text(
            serde_json::to_string(&self)
                .expect("Should never fail to serialize msg.")
                .into(),
        )
    }
}

impl ServerMsg {
    /// Name of the message
    ///
    /// Just a wrapper around [strum::IntoStaticStr] to help with typing
    pub fn name(&self) -> &'static str {
        self.into()
    }
}

/// Possible messages sent by the client
#[derive(Serialize, Deserialize, Debug, IntoStaticStr, PartialEq, Eq, EnumIs)]
#[serde(tag = "t", content = "c")]
pub enum ClientMsg {
    /// Request a new stylesheet
    ChangeCss {
        /// Which stylesheet to get
        index: i16,
        /// If the change is relative to the current css index
        relative: bool,
    },
    /// Client requests the server send [ServerMsg::Export]
    RequestExport,
    /// Request for the server to change the md file being viewed
    Redirect {
        /// Where the redirect is headed
        path: PathBuf,
    },
    /// Request for the server to change the md file being viewed back to the default
    RedirectDefault,
    /// Check that the server is running and responding to requests
    CheckServer,
}

impl AsMsg for ClientMsg {
    fn as_msg(&self) -> WsMessage {
        WsMessage::Text(
            serde_json::to_string(&self)
                .expect("Should never fail to serialize msg.")
                .into(),
        )
    }
}

impl ClientMsg {
    /// Name of the message
    ///
    /// Just a wrapper around [strum::IntoStaticStr] to help with typing
    pub fn name(&self) -> &'static str {
        self.into()
    }
}
