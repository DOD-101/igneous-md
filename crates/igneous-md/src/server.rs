//! Module containing items relating to the backend server.
//!
//! The server listens for incoming connections and upgrades them to websocket connections via
//! [crate::ws::upgrade_connection()].
//!
//! Each client connection is spawned as its own task, sharing a single [Config] between all clients.

use std::sync::{Arc, RwLock};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{config::Config, ws::upgrade_connection};

/// Handle to the running server
///
/// Dropping this handle will **not stop** the server. Use [Self::stop()] to stop the server.
pub struct ServerHandle {
    /// Channel to signal the server to stop
    stop_tx: oneshot::Sender<()>,
    /// Port the server is listening on
    port: u16,
}

impl ServerHandle {
    /// Get the port the server is listening on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Stop the server
    pub fn stop(self) -> Result<(), ()> {
        log::info!("Server exiting");
        self.stop_tx.send(())
    }
}

/// Launch the server
///
/// Binds to `127.0.0.1:{port}` and listens for incoming connections.
///
/// The server writes the port it is listening on to `/tmp/ingeous-md` since if `port` is 0 it will
/// randomly select a port to use.
pub async fn launch_server(
    port: u16,
    config: Config,
) -> Result<ServerHandle, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    let tcp_port = listener.local_addr()?.port();

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    if let Err(e) = std::fs::write("/tmp/ingeous-md", tcp_port.to_string()) {
        log::error!("Failed to write port to tmp file: {e}")
    };

    let config = Arc::new(RwLock::new(config));
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    return;
                }
                accept_result = listener.accept() => {
                    let (stream, _) = accept_result.expect("Failed to accept connection");
                    tokio::spawn(upgrade_connection(stream, Arc::clone(&config)));
                }
            }
        }
    });

    Ok(ServerHandle {
        stop_tx,
        port: tcp_port,
    })
}
