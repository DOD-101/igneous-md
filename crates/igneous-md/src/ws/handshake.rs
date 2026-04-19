//! WebSocket handshake and query parameter extraction.
//!
//! The handshake validates the incoming request query parameters and returns
//! a [WebSocketStream] along with the parsed [WsQueryParams].

// TODO: This requires improved logging

use http::{Request, Response};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};

/// Query parameters received during the WebSocket handshake.
#[derive(Debug)]
pub struct WsQueryParams {
    /// Interval in milliseconds between update checks.
    pub update_rate: Option<u64>,
    /// Path to the markdown file to serve.
    pub md_path: String,
}

/// Errors that can occur during WebSocket handshake validation.
#[derive(Debug, Copy, Clone, Error)]
pub enum WsValidationError {
    /// No query string was present in the request URI.
    #[error("Missing query string")]
    MissingQuery,
    /// The required `md_path` parameter was not found.
    #[error("Missing required parameter: md_path")]
    MissingMdPath,
}

impl WsQueryParams {
    /// Parse query parameters from an HTTP request.
    ///
    /// Expects `md_path` and optionally `update_rate` in the query string.
    pub fn from_request(request: &Request<()>) -> Result<Self, WsValidationError> {
        let query = request
            .uri()
            .query()
            .ok_or(WsValidationError::MissingQuery)?;

        let mut update_rate = None;
        let mut md_path = None;

        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            match key.as_ref() {
                "update_rate" => update_rate = value.parse::<u64>().ok(),
                "md_path" => md_path = Some(value.into_owned()),
                _ => {}
            }
        }

        let md_path = md_path.ok_or(WsValidationError::MissingMdPath)?;

        Ok(WsQueryParams {
            update_rate,
            md_path,
        })
    }
}

/// Create a callback for [accept_hdr_async] that validates query parameters.
///
/// The callback sends the parsed result through the oneshot channel and always
/// returns `Ok(response)`. Errors are communicated only via the channel.
#[allow(clippy::type_complexity)]
#[allow(
    clippy::result_large_err,
    reason = "Return type is required by callback parameter of tokio_tungstenite."
)]
pub fn ws_callback(
    sender: oneshot::Sender<Result<WsQueryParams, WsValidationError>>,
) -> impl FnOnce(&Request<()>, Response<()>) -> Result<Response<()>, Response<Option<String>>> {
    move |request, response| {
        let result = WsQueryParams::from_request(request);
        let _ = sender.send(result);
        Ok(response)
    }
}

/// Perform the WebSocket handshake on a TCP stream.
///
/// Returns the [WebSocketStream] and the validated [WsQueryParams] extracted
/// from the request query string.
pub async fn perform_handshake(
    tcp: TcpStream,
) -> Result<(WebSocketStream<TcpStream>, WsQueryParams), WsValidationError> {
    let (sender, receiver) = oneshot::channel();
    let callback = ws_callback(sender);

    let ws_stream = accept_hdr_async(tcp, callback)
        .await
        .map_err(|_| WsValidationError::MissingQuery)?;

    let params = receiver.await.expect("Callback must send result")?;

    Ok((ws_stream, params))
}
