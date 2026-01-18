//! WebSocket session management for Fansly API.

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::api::types::{WsAuthMessage, WsResponse, WsSessionData};
use crate::error::{Error, Result};

/// Fansly WebSocket URL.
const WS_URL: &str = "wss://wsv3.fansly.com";

/// Establish a WebSocket connection and obtain a session ID.
pub async fn get_session_id(token: &str) -> Result<String> {
    // Connect to WebSocket
    let (ws_stream, _) = connect_async(WS_URL).await?;
    let (mut write, mut read) = ws_stream.split();

    // Build auth message
    let auth_data = serde_json::json!({ "token": token }).to_string();
    let auth_msg = WsAuthMessage { t: 1, d: auth_data };
    let auth_json = serde_json::to_string(&auth_msg)?;

    // Send authentication message
    write.send(Message::Text(auth_json)).await?;

    // Read response
    while let Some(msg) = read.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            let response: WsResponse = serde_json::from_str(&text)?;

            // Type 0 indicates session response
            if response.t == 0 {
                let session_data: WsSessionData = serde_json::from_str(&response.d)?;
                return Ok(session_data.session.id);
            }
        }
    }

    Err(Error::Api("Failed to obtain session ID from WebSocket".into()))
}

#[cfg(test)]
mod tests {
    // WebSocket tests would require mocking or integration test setup
}
