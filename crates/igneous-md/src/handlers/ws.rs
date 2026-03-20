//! Module containing [upgrade_connection()] and all communication between client and server.
//!
//! Since we communicate everything via [Websockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
//! this is also where we handle that communication.
//!
//! Communication is done via json, which we [Serialize] using [serde_json]. See [ServerMsg] and
//! [ClientMsg].
use rocket::{
    futures::{SinkExt, StreamExt},
    tokio::{
        select,
        time::{self, Duration},
    },
    Shutdown, State,
};
use rocket_ws::{stream::DuplexStream, Channel, Message, WebSocket};
use std::{
    fs, io,
    sync::{Arc, Mutex},
};

mod msg;
use msg::{ClientMsg, ServerMsg};

use crate::{
    client::Client,
    config::Config,
    export::export,
    paths::{Paths, CONFIG_PATH},
};

/// Handles clients upgrading to Websocket
///
/// Once this succeeds the client is properly connected and will receive live-updates.
/// It will also now be able to request things from / communicate with the server via the Websocket.
///
/// Changing the viewed file is possible via [ClientMsgType::Redirect]
#[get("/ws?<update_rate>")]
pub async fn upgrade_connection(
    ws: WebSocket,
    paths: &State<Paths>,
    config: &State<Arc<Mutex<Config>>>,
    update_rate: Option<u64>,
    mut shutdown: Shutdown,
) -> io::Result<Channel<'static>> {
    let paths = paths.inner().clone();

    let mut client = Client::new(&paths, config.inner().clone());

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            // How often the`.md` file should be check for updates
            let mut interval = time::interval(Duration::from_millis(update_rate.unwrap_or(1000)));
            loop {
                select! {
                    // Check for updates in the`.md` file, sending any new HTML to the client
                    _ = interval.tick() => {
                        if let Ok(Some(html)) = client.get_latest_html_if_changed() {
                            log::info!("Sending new html");
                            let _ = stream.send_server_msg(
                                    &ServerMsg::HtmlUpdate { html }
                            ).await;
                        }
                    }

                    _ = client.config_update_receiver.recv() => {
                        log::info!("Sending update");
                        if let Some(css) = client.current_css() {
                            let css = fs::read_to_string(
                                CONFIG_PATH.join(
                                    css.strip_prefix("/").unwrap()));

                            let msg = match css {
                                Ok(css) => &ServerMsg::CssUpdate { css },
                                Err(e) => {
                                    let err = format!("Failed to send updated css after config update: {e}");

                                    &ServerMsg::Error { msg: err }
                                }
                            };

                            let _ = stream.send_server_msg(msg).await;
                        }
                    },

                    // Handle incoming messages from the client
                    incoming = stream.next() => {
                        match incoming {
                            Some(Ok(message)) => {
                                log::info!("Received ws message: {}", message);
                                match message {
                                    Message::Text(msg_string) => {
                                        if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&msg_string) {

                                            let return_msg = handle_client_msg(client_msg, &mut client, &paths);

                                            log::info!("Sending ws message: {:?}", return_msg);

                                            let _ = stream.send_server_msg(
                                                &return_msg
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
                    _  = &mut shutdown => {
                        let _ = stream.send_server_msg(&ServerMsg::Exit { error: false }).await;
                        break;
                    }
                }
            }
            Ok(())
        })
    }))
}

/// [upgrade_connection()] uses this to handle the incoming messages from the client
fn handle_client_msg(msg: ClientMsg, client: &mut Client, paths: &Paths) -> ServerMsg {
    match msg {
        ClientMsg::ChangeCss { index, relative } => {
            client.change_current_css_index(index, relative);

            if let Some(css) = client.current_css() {
                let current_css = CONFIG_PATH.join(css.strip_prefix("/").unwrap());

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
            if let Err(e) = export(client.get_html(), None) {
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
            client.set_md_path(paths.get_default_md());

            match client.get_latest_html() {
                Ok(html) => ServerMsg::HtmlUpdate { html },
                Err(e) => ServerMsg::Error { msg: e.to_string() },
            }
        }
    }
}

trait SendServerMsg {
    /// Convenience function to send a [ServerMsg] to the client
    ///
    /// Simply a wrapper around [rocket_ws::stream::DuplexStream::send()]
    fn send_server_msg(
        &mut self,
        msg: &ServerMsg,
    ) -> rocket::futures::sink::Send<'_, rocket_ws::stream::DuplexStream, rocket_ws::Message>;
}

impl SendServerMsg for DuplexStream {
    fn send_server_msg(
        &mut self,
        msg: &ServerMsg,
    ) -> rocket::futures::sink::Send<'_, rocket_ws::stream::DuplexStream, rocket_ws::Message> {
        self.send(Message::Text(serde_json::to_string(msg).expect("ERR")))
    }
}
