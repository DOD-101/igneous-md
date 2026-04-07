//! Module containing [upgrade_connection()] and all communication between client and server.
//!
//! Since we communicate everything via [Websockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
//! this is also where we handle that communication.
//!
//! Communication is done via json, which we [serde::Serialize] using [serde_json]. See [ServerMsg] and
//! [ClientMsg].

pub mod handshake;
pub mod msg;

use futures_util::{SinkExt, StreamExt};
use std::{
    io,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tokio::{
    net::TcpStream,
    sync::mpsc,
    time::{self, Duration},
};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::{client::Client, config::Config};
use handshake::perform_handshake;
use msg::{ClientMsg, ServerMsg};

/// Handles upgrading the connection to the Websocket protocol and facilitating communication
/// thereafter
pub async fn upgrade_connection(
    tcp: TcpStream,
    config: Arc<RwLock<Config>>,
    mut server_msg_rx: mpsc::UnboundedReceiver<ServerMsg>,
) -> io::Result<()> {
    let (ws_stream, params) = perform_handshake(tcp)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut client = Client::new(PathBuf::from(params.md_path), Arc::clone(&config));

    let (mut ws_write, mut ws_read) = ws_stream.split();

    let mut interval = time::interval(Duration::from_millis(params.update_rate.unwrap_or(1000)));

    // TODO: This should ideally be cleaned up (using a custom Stream type?). There are 4 different locations a message can be
    // sent from which can lead to inconsistencies in logging
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Ok(Some(html)) = client.get_latest_html_if_changed() {
                    let msg = ServerMsg::HtmlUpdate { html };
                    log::info!("Sending ws message: {}", msg.name());

                    let _ = ws_write.send(msg.as_msg()).await;
                }
            }

            _ = client.config_update_receiver.recv() => {
                if let Some(css) = client.current_css() {
                    let msg = ServerMsg::CssUpdate { css };
                    log::info!("Sending ws message: {}", msg.name());

                    let _ = ws_write.send(msg.as_msg()).await;
                }
            },

            Some(server_msg) = server_msg_rx.recv() => {
                log::info!("Sending msg from server backend: {}", server_msg.name());

                let _ = ws_write.send(server_msg.as_msg()).await;
            },

            incoming = ws_read.next() => {
                match incoming {
                    Some(Ok(message)) => {
                        match message {
                            WsMessage::Text(msg_string) => {
                                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg_string) {
                                    log::info!("Received ws message: {}", client_msg.name());
                                    log::debug!("Full received ws message: {:?}", client_msg);

                                    let return_msg = handle_client_msg(client_msg, &mut client);

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

    Ok(())
}

/// [upgrade_connection()] uses this to handle the incoming messages from the client
fn handle_client_msg(msg: ClientMsg, client: &mut Client) -> ServerMsg {
    match msg {
        ClientMsg::ChangeCss { index, relative } => {
            client.change_current_css_index(index, relative);

            if let Some(css) = client.current_css() {
                return ServerMsg::CssUpdate { css };
            }

            ServerMsg::Error {
                msg: "Failed to change css.".to_string(),
            }
        }
        ClientMsg::RequestExport => ServerMsg::Export {
            path: client.config.read().unwrap().export_path(),
        },
        ClientMsg::Redirect { path } => {
            client.set_md_path(path);

            match client.get_latest_html() {
                Ok(html) => ServerMsg::HtmlUpdate { html },
                Err(e) => ServerMsg::Error { msg: e.to_string() },
            }
        }
        ClientMsg::RedirectDefault => {
            client.reset_md_path_to_initial();

            match client.get_latest_html() {
                Ok(html) => ServerMsg::HtmlUpdate { html },
                Err(e) => ServerMsg::Error { msg: e.to_string() },
            }
        }
    }
}
