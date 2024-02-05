use futures::StreamExt;
use futures_util::{stream::SplitStream, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use tungstenite::error::{Error, ProtocolError};
use url::Url;

use crate::utils::errors::connect_error::ConnectError;

pub async fn connect_and_authenticate(
    uri: &str,
    auth_payload: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ConnectError> {
    match connect_async(Url::parse(uri).unwrap()).await {
        Ok((mut ws_stream, _)) => {
            ws_stream
                .send(Message::Text(auth_payload.to_string()))
                .await
                .unwrap();

            // Wait for an acknowledgment message
            match ws_stream.next().await {
                Some(Ok(message)) => {
                    println!("Received message: {:?}", message); // Log the message for debugging
                    match message {
                        Message::Text(ack_message) => {
                            if ack_message.contains("{\"result\":null,\"id\":0}") {
                                Ok(ws_stream) // Correct acknowledgment
                            } else {
                                Err(ConnectError::new(Error::Protocol(
                                    ProtocolError::InvalidCloseSequence,
                                )))
                            }
                        }
                        // Handle other message types if necessary
                        _ => Err(ConnectError::new(Error::Protocol(
                            ProtocolError::InvalidCloseSequence,
                        ))),
                    }
                }
                Some(Err(e)) => {
                    println!("Error receiving message: {:?}", e);
                    Err(ConnectError::new(Error::ConnectionClosed))
                }
                None => Err(ConnectError::new(Error::Protocol(
                    ProtocolError::InvalidCloseSequence,
                ))),
            }
        }
        Err(e) => {
            println!("Failed to connect: {:?}", e);
            Err(ConnectError::new(Error::ConnectionClosed))
        }
    }
}

pub async fn listen_to_messages(
    mut ws_reader: SplitStream<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    >,
) {
    while let Some(message) = ws_reader.next().await {
        match message {
            Ok(msg) => println!("Received message: {:?}", msg),
            Err(e) => {
                println!("Error: {:?}", e);
                break; // Exit loop on error
            }
        }
    }
}
