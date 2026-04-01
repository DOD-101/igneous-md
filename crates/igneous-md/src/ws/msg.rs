//! Messages sent by clients to the server and vice versa

// TODO: https://docs.rs/ts-rs/latest/ts_rs/
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

/// Struct representing a message from the server back to the client
#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
#[serde(tag = "t", content = "c")]
pub enum ServerMsg {
    /// Updated CSS for the html content
    ///
    /// The css can have changed for a variety of reasons.
    CssUpdate {
        /// Css content
        css: String,
        //NOTE: We could add a reason here in the future if there is a use
    },
    /// Updated HTML rendered from markdown
    HtmlUpdate {
        /// Html content
        html: String,
    },
    /// Server is shutting down
    ///
    /// There is no guarantee this message will be sent by the server. For example in the case of a
    /// panic.
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

impl ServerMsg {
    /// Convert [Self] into a [WsMessage]
    pub fn as_msg(&self) -> WsMessage {
        WsMessage::Text(
            serde_json::to_string(&self)
                .expect("Should never fail to serialize msg.")
                .into(),
        )
    }

    /// Name of the message
    ///
    /// Just a wrapper around [strum::IntoStaticStr] to help with typing
    pub fn name(&self) -> &'static str {
        self.into()
    }
}

/// Struct representing a message from the client
#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
#[serde(tag = "t", content = "c")]
pub enum ClientMsg {
    /// Request a new stylesheet
    ChangeCss {
        /// Which stylesheet to get
        ///
        /// To get the current stylesheet set this to `0`
        index: i16,
        /// If the change is relative to the current css index
        relative: bool,
    },
    /// Request for the server to export the html (save it to disk)
    ExportHtml,
    /// Request for the server to change the md file being viewed
    Redirect {
        /// Where the redirect is headed
        path: PathBuf,
    },
    /// Request for the server to change the md file being viewed back to the default
    RedirectDefault,
}

impl ClientMsg {
    /// Name of the message
    ///
    /// Just a wrapper around [strum::IntoStaticStr] to help with typing
    pub fn name(&self) -> &'static str {
        self.into()
    }
}
