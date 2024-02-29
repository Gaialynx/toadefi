use crate::config::CONFIG;
use crate::shared::utils::{
    eth_signer::EthSigner,
    websocket_utils::{connect_websocket, handle_websocket_messages},
};
use alloy_primitives::{Address, Uint};
use alloy_sol_types::Eip712Domain;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::borrow::Cow;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

use super::payload_signer::Signer;

#[derive(Debug)]
pub struct SubscriptionClient {
    signer: Signer, 
    needs_reconnect: Arc<AtomicBool>,
}

impl SubscriptionClient {
    pub fn new() -> Self {
        let eth_signer = EthSigner::new(&CONFIG.private_key);
        let domain = SubscriptionClient::create_domain().unwrap();

        let signer =Signer::new();

        SubscriptionClient {
            signer,
            needs_reconnect: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start_subscription(&self) -> Result<(), Box<dyn Error + Send>> {
        let subscribe_url = CONFIG.arbitrum_vertex_testnet_subscribe_url.clone();

        // Establish WebSocket connection
        let ws_stream = connect_websocket(&subscribe_url).await?;
        let (mut ws_writer, ws_reader) = ws_stream.split();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let expiration = current_time + 60_000; // Example expiration time set to 1 minute from now
        
        let signature = self.signer.sign_subscription_auth_payload(&CONFIG.sender_address);
        let ws_subscription_payload = json!({
            "method": "authenticate",
            "id": 0,
            "tx": {
                "sender": CONFIG.sender_address, // Use the sender address from your config
                "expiration": expiration, // Use the calculated expiration time
            },
            "signature": signature
        });
        
        // Send authentication payload (if needed) immediately after establishing the connection
        ws_writer
            .send(Message::Text(ws_subscription_payload.to_string()))
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

        // Listen to messages in a separate task
        tokio::spawn(async move {
            handle_websocket_messages(ws_reader).await;
        });

        // Start the ping task to keep the connection alive
        let needs_reconnect_clone = Arc::clone(&self.needs_reconnect);
        tokio::spawn(async move {
            SubscriptionClient::start_ping(ws_writer, needs_reconnect_clone).await;
        });

        Ok(())
    }

    async fn start_ping(
        mut ws_writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        needs_reconnect: Arc<AtomicBool>,
    ) {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if ws_writer.send(Message::Ping(Vec::new())).await.is_err() {
                println!("Ping failed, signaling reconnection");
                needs_reconnect.store(true, Ordering::Relaxed);
                break;
            }
        }
    }


    fn create_domain() -> Result<Eip712Domain, Box<dyn std::error::Error>> {
        let verifying_contract_bytes =
            hex::decode(CONFIG.arbitrum_testnet_contract.trim_start_matches("0x"))?;
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&verifying_contract_bytes);
        let verifying_contract = Address::from(bytes);

        let chain_id_value: Uint<256, 4> = Uint::from(CONFIG.arbitrum_testnet_chain_id);
        let chain_id = Some(chain_id_value);

        Ok(Eip712Domain {
            name: Some(Cow::Borrowed("Vertex")),
            version: Some(Cow::Borrowed("0.0.1")),
            chain_id,
            verifying_contract: Some(verifying_contract),
            salt: None,
        })
    }

    pub async fn check_and_reconnect(&self) {
        if self.needs_reconnect.load(Ordering::Relaxed) {
            // Connect to the WebSocket
            let ws_stream = connect_websocket(&CONFIG.arbitrum_vertex_testnet_subscribe_url)
                .await
                .expect("Failed to reconnect to WebSocket");

            // Split the WebSocket stream
            let (mut ws_writer, ws_reader) = ws_stream.split();

            // Reset the reconnection flag
            self.needs_reconnect.store(false, Ordering::Relaxed);

            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            let expiration = current_time + 60_000; // Example expiration time set to 1 minute from now
            
            let signature = self.signer.sign_subscription_auth_payload(&CONFIG.sender_address);
            let ws_subscription_payload = json!({
                "method": "authenticate",
                "id": 0,
                "tx": {
                    "sender": CONFIG.sender_address, // Use the sender address from your config
                    "expiration": expiration, // Use the calculated expiration time
                },
                "signature": signature
            });

            // Resend the authentication payload if necessary
            ws_writer
                .send(Message::Text(ws_subscription_payload.to_string()))
                .await
                .expect("Failed to send auth payload");

            // Spawn a task to listen to messages
            tokio::spawn(async move {
                handle_websocket_messages(ws_reader).await;
            });

            // Clone `needs_reconnect` for the ping task
            let needs_reconnect_clone = Arc::clone(&self.needs_reconnect);

            // Spawn a task for sending pings
            tokio::spawn(async move {
                SubscriptionClient::start_ping(ws_writer, needs_reconnect_clone).await;
            });
        }
    }
}
