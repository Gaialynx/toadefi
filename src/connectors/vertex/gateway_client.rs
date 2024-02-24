use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message;
use log::error; 

use crate::{
    config::CONFIG,
    shared::{errors::connect_error::ConnectError, utils::websocket_utils::connect_websocket},
};

#[derive(Debug, Default)]
pub struct GatewayClient {}

impl GatewayClient {
    pub fn new() -> Self {
        GatewayClient {}
    }

    pub async fn connect_to_gateway(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ConnectError> {
        // Use the generic connection function
        connect_websocket(&CONFIG.arbitrum_vertex_testnet_gateway_url).await
    }

    pub async fn send_message(&self, message: String) -> Result<String, ConnectError> {
        let ws_stream = match self.connect_to_gateway().await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to gateway: {}", e); // Log the error
                return Err(e);
            }
        };
    
        let (mut ws_writer, ws_reader) = ws_stream.split();
    
        // Send the query message
        if let Err(e) = ws_writer.send(Message::Text(message.clone())).await {
            error!("Failed to send message: {}", e); // Log the error
            return Err(ConnectError::new(e.into()));
        }
    
        // Listen for a response
        match self.await_response(ws_reader).await {
            Ok(response) => Ok(response),
            Err(e) => {
                error!("Error awaiting response: {}", e); // Log the error
                Err(e)
            }
        }
    }

    // Await a single response message
    async fn await_response(
        &self,
        mut ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<String, ConnectError> {
        match ws_reader.next().await {
            Some(response) => match response {
                Ok(msg) => match msg {
                    Message::Text(text) => Ok(text),
                    _ => {
                        let err = ConnectError::new(tungstenite::Error::AlreadyClosed);
                        error!("Received a non-text message: {}", err); // Log the error
                        Err(err)
                    }
                },
                Err(e) => {
                    let err = ConnectError::new(e.into());
                    error!("Error reading message: {}", err); // Log the error
                    Err(err)
                },
            },
            None => {
                let err = ConnectError::new(tungstenite::Error::AlreadyClosed);
                error!("WebSocket closed unexpectedly: {}", err); // Log the error
                Err(err)
            }
        }
    }
}
