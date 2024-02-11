use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use log::{error, info};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::shared::errors::connect_error::ConnectError;

/// Tries to establish a WebSocket connection with automatic reconnection on failure.
///
/// # Arguments
/// * `uri` - The WebSocket URL to connect to.
///
/// # Returns
/// A `WebSocketStream` wrapped in a `Result`, indicating success or a `ConnectError`.
pub async fn connect_websocket(
    uri: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ConnectError> {
    let url = Url::parse(uri).unwrap();
    let mut backoff = 1;

    loop {
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("WebSocket connection established.");
                return Ok(ws_stream);
            }
            Err(e) => {
                error!("Failed to connect: {:?}. Retrying in {}s", e, backoff);
                sleep(Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60); // Exponential backoff capped at 60 seconds
            }
        }
    }
}

/// Listens to messages from the WebSocket stream and handles them.
///
/// # Arguments
/// * `ws_stream` - The WebSocket stream to listen to.
///
/// # Note
/// This function demonstrates handling incoming WebSocket messages and errors in a robust manner,
/// suitable for a production environment.
pub async fn handle_websocket_messages(
    mut ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    while let Some(message) = ws_reader.next().await {
        match message {
            Ok(msg) => match msg {
                Message::Text(text) => info!("Received text message: {}", text),
                Message::Binary(bin) => info!("Received binary message: {:?}", bin),
                // Handle other message types as necessary
                _ => info!("Received other message."),
            },
            Err(e) => info!("Error receiving message: {:?}", e),
        }
    }
}
