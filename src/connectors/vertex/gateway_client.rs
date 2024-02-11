use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::protocol::Message;

use crate::{
    config::Config,
    shared::{errors::connect_error::ConnectError, utils::websocket_utils::connect_websocket},
};

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
        // Use the generic connection function
        connect_websocket(&self.config.arbitrum_vertex_testnet_gateway_url).await
    }

    pub async fn send_query_with_type(
        &self,
        query_message: String,
    ) -> Result<String, ConnectError> {
        let ws_stream = self.connect_to_gateway().await?;
        let (mut ws_writer, ws_reader) = ws_stream.split();

        // Send the query message
        ws_writer
            .send(Message::Text(query_message.clone()))
            .await
            .map_err(|e| ConnectError::new(e.into()))?;

        // Listen for a response
        let response = self.await_response(ws_reader).await?;

        Ok(response)
    }

    // Await a single response message
    async fn await_response(
        &self,
        mut ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<String, ConnectError> {
        if let Some(response) = ws_reader.next().await {
            match response {
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
