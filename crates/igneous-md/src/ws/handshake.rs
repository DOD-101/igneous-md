//! WebSocket handshake and query parameter extraction.
//!
//! The handshake validates the incoming request query parameters and returns
//! a [WebSocketStream] along with the parsed [WsQueryParams]. If validation
//! fails, the connector receives an HTTP 400 response explaining why.

use http::{Request, Response, StatusCode};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::{WebSocketStream, accept_hdr_async};

/// Query parameters received during the WebSocket handshake.
#[derive(Debug)]
pub struct WsQueryParams {
    /// Interval in milliseconds between update checks.
    pub update_rate: Option<u64>,
    /// Path to the markdown file to serve.
    pub md_path: String,
}

/// Errors that can occur while validating the handshake's query parameters.
#[derive(Debug, Copy, Clone, Error)]
pub enum WsValidationError {
    /// No query string was present in the request URI.
    #[error("Missing query string")]
    MissingQuery,
    /// The required `md_path` parameter was not found.
    #[error("Missing required parameter: md_path")]
    MissingMdPath,
    /// `update_rate` was present but not a valid, non-negative integer.
    #[error("Invalid value for parameter: update_rate")]
    InvalidUpdateRate,
}

/// Errors that can occur while performing a WebSocket handshake.
#[derive(Debug, Error)]
pub enum HandshakeError {
    /// The request's query parameters failed validation. The connector will
    /// have already received an HTTP 400 response explaining the problem.
    #[error("handshake query validation failed: {0}")]
    Validation(#[from] WsValidationError),
    /// The underlying WebSocket protocol handshake failed (e.g. bad
    /// upgrade headers, I/O error, connection reset).
    #[error("websocket handshake failed: {0}")]
    Protocol(#[from] WsError),
}

impl HandshakeError {
    /// Returns true if the error was caused by the client not wanting to upgrade to ws
    pub fn no_upgrade(&self) -> bool {
        matches!(
            self,
            HandshakeError::Protocol(WsError::Protocol(
                ProtocolError::MissingConnectionUpgradeHeader
            ))
        )
    }
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
                "update_rate" => {
                    let parsed = value
                        .parse::<u64>()
                        .map_err(|_| WsValidationError::InvalidUpdateRate)?;
                    update_rate = Some(parsed);
                }
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

/// Build the HTTP error response sent back to the connector when query
/// validation fails, so it fails the handshake with a real 400 instead of
/// silently upgrading to a socket that will be immediately closed.
fn rejection_response(err: WsValidationError) -> Response<Option<String>> {
    let mut response = Response::new(Some(err.to_string()));
    *response.status_mut() = StatusCode::BAD_REQUEST;
    response
}

/// Perform the WebSocket handshake on a TCP stream.
///
/// Returns the [WebSocketStream] and the validated [WsQueryParams] extracted
/// from the request query string. If the query parameters are invalid, the
/// connector receives an HTTP 400 response during the handshake itself
/// (rather than a socket that opens and is then torn down) and this
/// function returns [HandshakeError::Validation]. Any lower-level protocol
/// or I/O failure is returned as [HandshakeError::Protocol].
#[allow(
    clippy::result_large_err,
    reason = "Return type is required by callback parameter of tokio_tungstenite."
)]
pub async fn perform_handshake(
    tcp: TcpStream,
) -> Result<(WebSocketStream<TcpStream>, WsQueryParams), HandshakeError> {
    let mut params: Option<Result<WsQueryParams, WsValidationError>> = None;

    let callback = |request: &Request<()>, response: Response<()>| match WsQueryParams::from_request(
        request,
    ) {
        Ok(parsed) => {
            params = Some(Ok(parsed));
            Ok(response)
        }
        Err(err) => {
            params = Some(Err(err));
            Err(rejection_response(err))
        }
    };

    let ws_stream = accept_hdr_async(tcp, callback).await?;

    // If accept_hdr_async succeeded, the callback ran and returned `Ok`,
    // so `params` is always `Some(Ok(_))` here.
    let params = params.expect("callback must run before accept_hdr_async resolves")?;

    Ok((ws_stream, params))
}
