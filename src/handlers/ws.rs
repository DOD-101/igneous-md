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

use crate::client::Client;

#[derive(Deserialize)]
struct ClientMsg {
    r#type: ClientMsgType,
}

#[derive(Deserialize)]
enum ClientMsgType {
    ChangeCssNext,
    ChangeCssPrev,
}

#[derive(Serialize)]
struct ServerMsg {
    r#type: ServerMsgType,
    body: String,
}

#[derive(Serialize)]
enum ServerMsgType {
    CssUpdate,
    HtmlUpdate,
}

/// Handles clients upgrading to websocket to receive file updates
///
/// This function will upgrade the connection to websocket and spawn a new thread for the
/// connection.
#[get("/ws?<path>")]
pub async fn upgrade_connection(
    ws: WebSocket,
    path: &str,
    config_dir: &State<PathBuf>,
) -> Result<Channel<'static>, io::Error> {
    let mut client = match Client::new(PathBuf::from(".").join(path), config_dir.to_path_buf()) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to init client with path: {} Error: {}", path, e);
            return Err(e);
        }
    };

    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                select! {
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

fn handle_client_msg(msg: ClientMsg, client: &mut Client) -> ServerMsg {
    match msg.r#type {
        ClientMsgType::ChangeCssNext => {
            let path = client.config.next_css().expect("Failed to get next css");

            ServerMsg {
                r#type: ServerMsgType::CssUpdate,
                body: path.to_string_lossy().to_string(),
            }
        }
        ClientMsgType::ChangeCssPrev => {
            let path = client
                .config
                .previous_css()
                .expect("Failed to get previous css");

            ServerMsg {
                r#type: ServerMsgType::CssUpdate,
                body: path.to_string_lossy().to_string(),
            }
        }
    }
}
