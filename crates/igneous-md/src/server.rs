//! Module for the backend server
//!
//! The server accepts new clients and sends communicates with them via message passing.
//!
//! Each client connection is spawned as its own task, sharing a single [Config] between all clients.
//!
//! [`Client`]s are not directly in the server, but rather their [`ClientHandle`] is tracked.
use futures_util::{SinkExt, StreamExt};
use notify::{INotifyWatcher, Watcher};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::{JoinError, JoinSet},
};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::{
    client::{Client, ClientHandle, ClientInputMsg},
    config::Config,
    paths,
    ws::{
        handshake::{HandshakeError, perform_handshake},
        msg::{AsMsg, ClientMsg, ServerMsg},
    },
};

/// Server facilitating communication to the clients
#[derive(Debug)]
pub struct Server {
    /// Sender for messages to the server
    input_tx: mpsc::UnboundedSender<ServerInputMsg>,
    /// Receiver for messages to the server
    input_rx: mpsc::UnboundedReceiver<ServerInputMsg>,

    /// Shared config for the application
    config: Arc<Config>,
    /// ClientHandles and their associated ids
    clients: HashMap<u16, ClientHandle>,
    /// JoinSet for tasks for clients
    client_tasks: JoinSet<u16>,
}

impl Server {
    /// Creates a new [`Server`].
    pub fn new(config: Arc<Config>) -> Self {
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        Self {
            input_tx,
            input_rx,
            config,
            clients: HashMap::with_capacity(1),
            client_tasks: JoinSet::default(),
        }
    }

    /// Start the server
    pub async fn start(mut self, port: u16) {
        let listener = match TcpListener::bind(format!("127.0.0.1:{port}")).await {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to bind tcp socket with port {port}: {e}");
                return;
            }
        };

        paths::attempt_write_port_file(port);

        let _watcher = match self.watch_config(&self.config.config_dir) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Failed to watch config dir: {e}");
                return;
            }
        };

        let mut client_id = 0;
        loop {
            tokio::select! {
                Ok(accept_result) = listener.accept() => self.handle_accept(&mut client_id, accept_result.0).await,
                Some(input_msg) = self.input_rx.recv() => self.handle_input(input_msg),
                Some(exited_res) = self.client_tasks.join_next(), if !self.client_tasks.is_empty()
                                    => self.handle_client_exit(exited_res),
            }
        }
    }

    /// Helper method for accepting new clients
    async fn handle_accept(&mut self, client_id: &mut u16, stream: TcpStream) {
        // we could make this truly async if the need arises
        match tokio::time::timeout(
            Duration::from_secs(1),
            self.spawn_client(stream, Arc::clone(&self.config), *client_id),
        )
        .await
        {
            Ok(Ok(_)) => (),
            Err(e) => {
                log::error!("Upgrading to ws connection timed out: {e}");
                return;
            }
            Ok(Err(e)) => {
                if !e.no_upgrade() {
                    log::error!("Failed to upgrade connection to ws: {e}");
                } else {
                    log::debug!("Attempted non-ws upgrade connection.")
                }
                return;
            }
        };

        // only increment the client id if we actually spawned a client
        *client_id += 1;
    }

    /// Spawns a new client on a new Websocket connection
    async fn spawn_client(
        &mut self,
        tcp: TcpStream,
        config: Arc<Config>,
        id: u16,
    ) -> Result<(), HandshakeError> {
        let (ws_stream, params) = perform_handshake(tcp).await?;

        let (client, client_handle) =
            Client::new(id, PathBuf::from(&params.md_path), Arc::clone(&config));

        self.client_tasks.spawn(client.start(ws_stream, params));
        self.clients.insert(id, client_handle);

        Ok(())
    }

    /// Helper method for handling input messages
    fn handle_input(&mut self, msg: ServerInputMsg) {
        match msg {
            ServerInputMsg::ConfigUpdate(config) => {
                let config = Arc::from(config);

                for ch in self.clients.values_mut() {
                    if let Err(e) = ch
                        .input_tx
                        .try_send(ClientInputMsg::ConfigUpdate(Arc::clone(&config)))
                    {
                        log::warn!("Failed to send updated config to client: {e}")
                    }
                }
            }
        }
    }

    /// Helper method for dealing with clients exiting / disconnecting
    fn handle_client_exit(&mut self, exited_res: Result<u16, JoinError>) {
        let id = match exited_res {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Client panicked: {e}");
                return;
            }
        };

        self.clients.remove(&id);
    }

    /// Start watching the `config_dir`
    ///
    /// After this will start sending events.
    pub fn watch_config(&self, config_dir: &Path) -> notify::Result<INotifyWatcher> {
        let mut watcher = notify::recommended_watcher({
            let config_dir = config_dir.to_path_buf();
            let sender = self.input_tx.clone();

            move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event
                    && !event.kind.is_access()
                {
                    log::info!("Config update");
                    let new_config = match Config::new_from_disk(config_dir.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            log::error!("Failed to create a new config: {e}");
                            return;
                        }
                    };
                    let _ = sender.send(ServerInputMsg::ConfigUpdate(new_config));
                }
            }
        })?;

        log::info!("Watching config dir: {}", config_dir.to_string_lossy());

        watcher
            .watch(config_dir, notify::RecursiveMode::Recursive)
            .unwrap();

        Ok(watcher)
    }
}

/// Input messages to the server
#[derive(Debug, Clone)]
enum ServerInputMsg {
    /// Config updated
    ConfigUpdate(Config),
}

/// Test if there is a server running on the given port
pub async fn test_server_connection(port: u16) -> bool {
    let Ok((mut socket, _)) =
        tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/ws/?md_path=/dev/null"))
            .await
    else {
        return false;
    };

    socket.send(ClientMsg::CheckServer.as_msg()).await.unwrap();

    socket.select_next_some().await.is_ok_and(|msg| {
        if let WsMessage::Text(str) = msg {
            return serde_json::from_str::<ServerMsg>(&str).is_ok_and(|v| v.is_success());
        }

        false
    })
}
