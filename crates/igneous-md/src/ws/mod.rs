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
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tokio::{
    net::TcpStream,
    time::{self, Duration},
};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use self::handshake::perform_handshake;
use crate::{client::Client, config::Config, export::export};
use msg::{ClientMsg, ServerMsg};

/// Handles upgrading the connection to the Websocket protocol and facilitating communication
/// thereafter
pub async fn upgrade_connection(tcp: TcpStream, config: Arc<RwLock<Config>>) -> io::Result<()> {
    let (ws_stream, params) = perform_handshake(tcp)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut client = Client::new(PathBuf::from(params.md_path), Arc::clone(&config));

    let (mut ws_write, mut ws_read) = ws_stream.split();

    let mut interval = time::interval(Duration::from_millis(params.update_rate.unwrap_or(1000)));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Ok(Some(html)) = client.get_latest_html_if_changed() {
                    log::info!("Sending new html");
                    let msg = ServerMsg::HtmlUpdate { html };
                    if let Ok(text) = serde_json::to_string(&msg) {
                        let _ = ws_write.send(WsMessage::Text(text.into())).await;
                    }
                }
            }

            _ = client.config_update_receiver.recv() => {
                log::info!("Sending config update");
                if let Some(css) = client.current_css() {
                    let css = fs::read_to_string(
                        client.config.read().unwrap().config_dir().join(
                            css.strip_prefix("/").unwrap()));

                    let msg = match css {
                        Ok(css) => ServerMsg::CssUpdate { css },
                        Err(e) => {
                            let err = format!("Failed to send updated css after config update: {e}");
                            ServerMsg::Error { msg: err }
                        }
                    };

                    if let Ok(text) = serde_json::to_string(&msg) {
                        let _ = ws_write.send(WsMessage::Text(text.into())).await;
                    }
                }
            },

            incoming = ws_read.next() => {
                match incoming {
                    Some(Ok(message)) => {
                        log::info!("Received ws message: {}", message);
                        match message {
                            WsMessage::Text(msg_string) => {
                                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg_string) {
                                    let return_msg = handle_client_msg(client_msg, &mut client);
                                    log::info!("Sending ws message: {:?}", return_msg);
                                    if let Ok(text) = serde_json::to_string(&return_msg) {
                                        let _ = ws_write.send(WsMessage::Text(text.into())).await;
                                    }
                                } else {
                                    log::warn!("Invalid client message: {}", msg_string)
                                }
                            },

                            WsMessage::Close(_) => {
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
}

/// [upgrade_connection()] uses this to handle the incoming messages from the client
fn handle_client_msg(msg: ClientMsg, client: &mut Client) -> ServerMsg {
    match msg {
        ClientMsg::ChangeCss { index, relative } => {
            client.change_current_css_index(index, relative);

            if let Some(css) = client.current_css() {
                let current_css = client
                    .config
                    .read()
                    .unwrap()
                    .config_dir()
                    .join(css.strip_prefix("/").unwrap());

                let css = fs::read_to_string(current_css);

                if let Ok(css) = css {
                    return ServerMsg::CssUpdate { css };
                }
            }

            ServerMsg::Error {
                msg: "Failed to change css.".to_string(),
            }
        }
        ClientMsg::ExportHtml => {
            if let Err(e) = export(
                client.get_html(),
                client.config.read().unwrap().config_dir(),
                None,
            ) {
                ServerMsg::Error { msg: e.to_string() }
            } else {
                ServerMsg::Success
            }
        }
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
