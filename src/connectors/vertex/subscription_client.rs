use super::utils::signer::Signer;
use super::vertex_auth::VertexAuth;
use crate::config::Config;
use crate::connectors::shared::websocket::{connect_and_authenticate, listen_to_messages};
use alloy_primitives::{Address, Uint};
use alloy_sol_types::Eip712Domain;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::borrow::Cow;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Default)]
pub struct SubscriptionClient {
    config: Config,
    ws_subscription_payload: String,
    connection: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    needs_reconnect: Arc<AtomicBool>,
}

impl SubscriptionClient {
    pub fn new(config: Config) -> Self {
        let vertex_auth = VertexAuth::new(&config.private_key);
        let domain = SubscriptionClient::create_domain(&config).unwrap();

        let signer =
            SubscriptionClient::create_signer(config.sender_address.clone(), &vertex_auth, &domain)
                .unwrap();
        let ws_subscription_payload = signer.construct_ws_auth_payload().unwrap();

        SubscriptionClient {
            config,
            ws_subscription_payload,
            connection: Arc::new(Mutex::new(None)),
            needs_reconnect: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start_subscription(&self) -> Result<(), Box<dyn Error + Send>> {
        let subscribe_url = self.config.arbitrum_vertex_testnet_subscribe_url.clone();

        let mut conn_guard = self.connection.lock().await;
        if let Some(mut existing_conn) = conn_guard.take() {
            let close_msg = Message::Close(None);
            if let Err(e) = existing_conn.send(close_msg).await {
                println!("Error sending close message: {:?}", e);
            }
        }

        let ws_stream =
            connect_and_authenticate(&subscribe_url, self.ws_subscription_payload.as_str()).await?;
        let (ws_writer, ws_reader) = ws_stream.split();

        tokio::spawn(async move {
            listen_to_messages(ws_reader).await;
        });

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

    fn create_signer<'a>(
        sender_address: String,
        vertex_auth: &'a VertexAuth,
        domain: &'a Eip712Domain,
    ) -> Result<Signer<'a>, Box<dyn std::error::Error>> {
        Ok(Signer::new(sender_address, vertex_auth, domain))
    }

    fn create_domain(config: &Config) -> Result<Eip712Domain, Box<dyn std::error::Error>> {
        let verifying_contract_bytes =
            hex::decode(config.arbitrum_testnet_contract.trim_start_matches("0x"))?;
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&verifying_contract_bytes);
        let verifying_contract = Address::from(bytes);

        let chain_id_value: Uint<256, 4> = Uint::from(config.arbitrum_testnet_chain_id);
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
            let ws_stream = connect_and_authenticate(
                &self.config.arbitrum_vertex_testnet_subscribe_url,
                &self.ws_subscription_payload,
            )
            .await
            .unwrap();
            let (ws_writer, ws_reader) = ws_stream.split();

            self.needs_reconnect.store(false, Ordering::Relaxed);

            tokio::spawn(async move {
                listen_to_messages(ws_reader).await;
            });

            let needs_reconnect_clone = Arc::clone(&self.needs_reconnect);
            tokio::spawn(async move {
                SubscriptionClient::start_ping(ws_writer, needs_reconnect_clone).await;
            });
        }
    }
}
