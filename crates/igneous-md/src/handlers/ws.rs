//! Module containing [upgrade_connection()] and all communication between client and server.
//!
//! Since we communicate everything via [Websockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
//! this is also where we handle that communication.
//!
//! Communication is done via json, which we [Serialize] using [serde_json]. See [ServerMsg] and
//! [ClientMsg].
use rocket::{
    futures::{SinkExt, StreamExt},
    serde::{Deserialize, Serialize},
    tokio::{
        select,
        time::{self, Duration},
    },
    State,
};
use rocket_ws::{Channel, Message, WebSocket};
use std::{io, path::PathBuf};

use crate::{client::Client, export::export, paths::Paths};

/// Struct representing a message from the client
#[derive(Deserialize, Debug)]
struct ClientMsg {
    /// The type of message
    r#type: ClientMsgType,
}

/// Different types of messages the client can send
#[derive(Deserialize, Debug)]
enum ClientMsgType {
    /// Request the next css file. See [Client::config]
    ChangeCssNext,
    /// Request the previous css file. See [Client::config]
    ChangeCssPrev,
    /// Request for the server to export the html (save it to disk)
    ExportHtml,
}

/// Struct representing a message from the server back to the client
#[derive(Serialize, Debug)]
struct ServerMsg {
    /// The type of message
    r#type: ServerMsgType,
    /// The content of the message
    body: String,
}

impl ServerMsg {
    /// A convenience function to create a [ServerMsg] with type [ServerMsgType::Success]
    fn success() -> Self {
        Self {
            r#type: ServerMsgType::Success,
            body: String::new(),
        }
    }

    /// A convenience function to create a [ServerMsg] with type [ServerMsgType::Success]
    fn error(msg: String) -> Self {
        Self {
            r#type: ServerMsgType::Error,
            body: msg,
        }
    }
}

/// Different types of messages the server can send
#[derive(Serialize, Debug)]
enum ServerMsgType {
    /// A message telling the client to treat the body as a new css file path
    CssUpdate,
    /// A message telling the client to treat the body as the new content of the `<body>` element
    HtmlUpdate,
    /// An arbitrary success message
    Success,
    /// An arbitrary error message
    Error,
}

/// Handles clients upgrading to Websocket
///
/// Once this succeeds the client is properly connected and will receive live-updates for the
/// `path` (the md file). It will also now be able to request things from / communicate with
/// the server via the Websocket.
#[get("/ws?<path>")]
pub async fn upgrade_connection(
    ws: WebSocket,
    path: &str,
    paths: &State<Paths>,
) -> io::Result<Channel<'static>> {
    let mut client = match Client::new(PathBuf::from(".").join(path), paths.inner().clone()) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to init client with path: {} Error: {}", path, e);
            return Err(e);
        }
    };

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            // How often the`.md` file should be check for updates
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                select! {
                    // Check for updates in the`.md` file, sending any new HTML to the client
                    _ = interval.tick()=> {
                        if let Ok(Some(html)) = client.get_latest_html_if_changed() {
                            log::info!("Sending new html");
                            let _ = stream.send(
                                Message::Text(
                                    serde_json::to_string(&ServerMsg {
                                        r#type: ServerMsgType::HtmlUpdate,
                                        body: html,
                                    })
                                    .expect("Failed to turn ServerMsg into json"),
                                )
                            ).await;
                        }
                    }

                    // Handle incoming messages from the client
                    incoming = stream.next() => {
                        match incoming {
                            Some(Ok(message)) => {
                                log::info!("Received ws message: {}", message);
                                match message {
                                    Message::Text(msg_string) => {
                                        if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg_string) {

                                            let return_msg = handle_client_msg(client_msg, &mut client);

                                            log::info!("Sending ws message: {:?}", return_msg);

                                            let _ = stream.send(
                                                Message::Text(serde_json::to_string(&return_msg)
                                                              .expect("Failed to turn ServerMsg into json"))
                                            ).await;
                                        } else {
                                            log::warn!("Invalid client Message: {}", msg_string)
                                        }
                                    },

                                    Message::Close(_) => {
                                        log::info!("Client initiated connection close");
                                        break;
                                    }
                                    _ => {}
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
            Ok(())
        })
    }))
}

/// [upgrade_connection()] uses this to handle the incoming messages from the client
fn handle_client_msg(msg: ClientMsg, client: &mut Client) -> ServerMsg {
    match msg.r#type {
        ClientMsgType::ChangeCssNext => {
            let path = client.config.next_css();

            if let Some(path) = path {
                ServerMsg {
                    r#type: ServerMsgType::CssUpdate,
                    body: path.to_string_lossy().to_string(),
                }
            } else {
                ServerMsg::error("No css files provided".to_string())
            }
        }
        ClientMsgType::ChangeCssPrev => {
            let path = client.config.previous_css();

            if let Some(path) = path {
                ServerMsg {
                    r#type: ServerMsgType::CssUpdate,
                    body: path.to_string_lossy().to_string(),
                }
            } else {
                ServerMsg::error("No css files provided".to_string())
            }
        }
        ClientMsgType::ExportHtml => {
            if let Err(e) = export(client.get_html(), None) {
                ServerMsg::error(e.to_string())
            } else {
                ServerMsg::success()
            }
        }
    }
}
