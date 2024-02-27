use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub sender_address: String,
    pub private_key: String,
    pub arbitrum_testnet_contract: String,
    pub arbitrum_testnet_chain_id: i32,
    pub arbitrum_vertex_testnet_subscribe_url: String,
    pub arbitrum_vertex_testnet_gateway_url: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();

        Self {
            sender_address: env::var("SENDER_ADDRESS").expect("SENDER_ADDRESS not set"),
            private_key: env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"),
            arbitrum_testnet_chain_id: env::var("ARBITRUM_TESTNET_CHAIN_ID")
                .expect("ARBITRUM_TESTNET_CHAIN_ID not set")
                .parse()
                .expect("ARBITRUM_TESTNET_CHAIN_ID must be an integer"),
            arbitrum_testnet_contract: env::var("ARBITRUM_TESTNET_CONTRACT")
                .expect("ARBITRUM_TESTNET_CONTRACT not set"),
            arbitrum_vertex_testnet_subscribe_url: env::var(
                "ARBITRUM_VERTEX_TESTNET_SUBSCRIBE_URL",
            )
            .expect("ARBITRUM_VERTEX_TESTNET_SUBSCRIBE_URL not set"),
            arbitrum_vertex_testnet_gateway_url: env::var("ARBITRUM_VERTEX_TESTNET_GATEWAY_URL")
                .expect("ARBITRUM_VERTEX_TESTNET_GATEWAY_URL is not set"),
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
