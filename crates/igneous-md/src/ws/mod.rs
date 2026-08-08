//! Module containing [upgrade_connection()] and all communication between client and server.
//!
//! Since we communicate everything via [Websockets](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
//! this is also where we handle that communication.
//!
//! Communication is done via json, which we [serde::Serialize] using [serde_json]. See [ServerMsg] and
//! [ClientMsg].

pub mod handshake;
pub mod msg;
