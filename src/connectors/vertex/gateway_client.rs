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

    pub async fn send_query_with_type(
        &self,
        query_message: String,
    ) -> Result<String, ConnectError> {
        let mut ws_stream = self.connect_to_gateway().await?;
        // Construct and send the query based on the query_type argument
        self.send_and_receive(&mut ws_stream, &query_message).await
    }

    // pub async fn place_order(&self, query_type:&str) -> Result<String, ConnectError> {
    //     let mut ws_stream= self.connect_to_gateway().await?;

    //     // get product id
    //     // sign payload
    //     // Construct and send place order payload
    //     let query_message = serde_json::json!({

    //     }).to_string();

    //     ws_stream
    //     .send(Message::Text(place_order_message)
    // }

    // send payload or recieve response from websocket
    async fn send_and_receive(
        &self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message: &str,
    ) -> Result<String, ConnectError> {
        ws_stream
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|e| ConnectError::new(e.into()))?;

        if let Some(response) = ws_stream.next().await {
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
