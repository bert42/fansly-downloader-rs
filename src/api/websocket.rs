//! WebSocket session management for Fansly API.

use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{handshake::client::Request, Message},
};

use crate::api::types::WsSessionData;
use crate::error::{Error, Result};

/// Fansly WebSocket URL.
const WS_URL: &str = "wss://wsv3.fansly.com";

/// WebSocket connection timeout.
const WS_TIMEOUT: Duration = Duration::from_secs(10);

/// Establish a WebSocket connection and obtain a session ID.
pub async fn get_session_id(token: &str, user_agent: &str) -> Result<String> {
    // Build request with required headers
    let request = Request::builder()
        .uri(WS_URL)
        .header("User-Agent", user_agent)
        .header("Origin", "https://fansly.com")
        .header("Host", "wsv3.fansly.com")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .body(())
        .map_err(|e| Error::Api(format!("Failed to build WebSocket request: {}", e)))?;

    // Connect to WebSocket with headers
    let (ws_stream, _) = connect_async(request).await?;
    let (mut write, mut read) = ws_stream.split();

    // Build auth message - format must match exactly what Fansly expects
    // The 'd' field is a JSON string containing the token object
    // Result: {"t":1,"d":"{\"token\":\"TOKEN\"}"}
    let inner = format!(r#"{{"token":"{}"}}"#, token);
    let escaped_inner = inner.replace('"', r#"\""#);
    let auth_json = format!(r#"{{"t":1,"d":"{}"}}"#, escaped_inner);
    tracing::debug!("Sending WebSocket auth message: {}", auth_json);

    // Send authentication message
    write.send(Message::Text(auth_json)).await?;

    // Read response with timeout
    let response = timeout(WS_TIMEOUT, read.next())
        .await
        .map_err(|_| Error::Api("WebSocket response timeout".into()))?
        .ok_or_else(|| Error::Api("WebSocket closed without response".into()))??;

    tracing::debug!("WebSocket response: {:?}", response);

    if let Message::Text(text) = response {
        // Parse the response
        let response: serde_json::Value = serde_json::from_str(&text)?;
        tracing::debug!("Parsed response: {:?}", response);

        let t = response["t"].as_i64().unwrap_or(-1);

        // Type 0 is an error
        if t == 0 {
            return Err(Error::Api(format!("WebSocket auth error: {}", text)));
        }

        // Type 1 or other - parse session data from 'd' field
        let d = response["d"]
            .as_str()
            .ok_or_else(|| Error::Api("Missing 'd' field in WebSocket response".into()))?;

        let session_data: WsSessionData = serde_json::from_str(d)?;
        return Ok(session_data.session.id);
    }

    Err(Error::Api("Unexpected WebSocket response type".into()))
}

#[cfg(test)]
mod tests {
    // WebSocket tests would require mocking or integration test setup
}
