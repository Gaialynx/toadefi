use futures_util::{SinkExt, StreamExt}; // used by ws_stream to send
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::{config::Config, utils::errors::connect_error::ConnectError};

#[derive(Debug, Default)]
pub struct GatewayClient {
    config: Config,
}

impl GatewayClient {
    pub fn new(config: Config) -> Self {
        GatewayClient { config }
    }

    pub async fn connect_to_gateway(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ConnectError> {
        let (ws_stream, _) = connect_async(&self.config.arbitrum_vertex_testnet_gateway_url)
            .await
            .unwrap();
        Ok(ws_stream)
    }

    pub async fn send_query(&self, query_type: &str) -> Result<String, ConnectError> {
        let mut ws_stream = self.connect_to_gateway().await?;

        // Construct and send the query
        let query_message = serde_json::json!({ "type": query_type }).to_string();
        ws_stream
            .send(Message::Text(query_message))
            .await
            .map_err(|e| ConnectError::new(e.into()))?;

        // Await the response
        if let Some(message) = ws_stream.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(text) => Ok(text),
                    _ => Err(ConnectError::new(tungstenite::Error::AlreadyClosed)),
                },
                Err(e) => Err(ConnectError::new(e.into())),
            }
        } else {
            Err(ConnectError::new(tungstenite::Error::AlreadyClosed))
        }
    }
}
