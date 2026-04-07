//! Module containing items relating to the backend server.
//!
//! The server listens for incoming connections and upgrades them to websocket connections via
//! [crate::ws::upgrade_connection()].
//!
//! Each client connection is spawned as its own task, sharing a single [Config] between all clients.

use std::sync::{Arc, RwLock};
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

use crate::{
    config::Config,
    ws::{msg::ServerMsg, upgrade_connection},
};

/// Handle to the running server
///
/// Dropping this handle will **not stop** the server. Use [Self::stop()] to stop the server.
pub struct ServerHandle {
    /// Channel to signal the server to stop
    stop_tx: oneshot::Sender<()>,
    /// Port the server is listening on
    port: u16,
    /// Senders to clients which have connected
    ///
    /// This allows the server to send messages to clients.
    ///
    /// Beware that after clients disconnect their senders remain here until the server is
    /// shutdown. This means that sending a message to a client won't always be successful, since
    /// the other side may have been dropped.
    ///
    // TODO: In the future we should replace this with a proper ClientHandle struct so we have more
    // control over the clients. This will also allow us to clean up the Action::convert code.
    clients: Arc<RwLock<Vec<mpsc::UnboundedSender<ServerMsg>>>>,
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

    /// Get the client sender at `index`
    ///
    /// The returned value is cloned.
    pub fn get_client_sender(&self, index: usize) -> Option<mpsc::UnboundedSender<ServerMsg>> {
        self.clients
            .read()
            .expect("Clients RWLock should never be poisoned.")
            .get(index)
            .cloned()
    }
}

/// Launch the server
///
/// Binds to `127.0.0.1:{port}` and listens for incoming connections.
///
/// The server writes the port it is listening on to `/tmp/ingeous-md` since if `port` is 0 it will
/// randomly select a port to use.
pub async fn launch_server(port: u16, config: Config) -> Result<ServerHandle, std::io::Error> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    let tcp_port = listener.local_addr()?.port();

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    if let Err(e) = std::fs::write("/tmp/ingeous-md", tcp_port.to_string()) {
        log::error!("Failed to write port to tmp file: {e}")
    };

    let config = Arc::new(RwLock::new(config));
    let clients: Arc<RwLock<Vec<mpsc::UnboundedSender<ServerMsg>>>> = Arc::default();

    let clients_clone = Arc::clone(&clients);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    return;
                }
                accept_result = listener.accept() => {
                    let (stream, _) = accept_result.expect("Failed to accept connection");
                    let (tx, rx) = mpsc::unbounded_channel();

                    tokio::spawn(upgrade_connection(stream, Arc::clone(&config), rx));

                    clients_clone.write().expect("Clients RWLock should never be poisoned.").push(tx);
                }
            }
        }
    });

    Ok(ServerHandle {
        stop_tx,
        port: tcp_port,
        clients: Arc::clone(&clients),
    })
}
