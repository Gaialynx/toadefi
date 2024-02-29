use crate::config::CONFIG;
use crate::domain::models::vertex::sol_structs::{Order, StreamAuthentication};
use crate::shared::utils::eth_signer::EthSigner;
use crate::shared::utils::type_conv;
use alloy_primitives::{Address, Uint};
use alloy_sol_types::{Eip712Domain, SolStruct};
use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Signer {
    eth_signer:  EthSigner,
    domain:  Eip712Domain,
}

impl Signer {
    pub fn new(
    ) -> Self {
        let eth_signer = EthSigner::new(&CONFIG.private_key);
        let domain = create_domain().unwrap();
        Signer {
            eth_signer: eth_signer,
            domain: domain,
        }
    }

    pub fn sign_subscription_auth_payload(&self,  sender_address:&str) -> String{
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let expiration = current_time + 60_000; // Expiration time set to 1 minute from now

    
        // Initialize StreamAuthentication using the abstracted method for generating sender bytes
        let tx_data = StreamAuthentication {
            sender: type_conv::hex_to_fixed_bytes32(sender_address).unwrap(),
            expiration,
        };
        
        let signing_hash = tx_data.eip712_signing_hash(&self.domain);
        let signature = self
        .eth_signer
        .generate_signature(signing_hash.as_ref()).unwrap();

        signature
    }

    pub fn sign_place_order_payload(&self,  order:&Order) -> String{
        let signing_hash = order.eip712_signing_hash(&self.domain);
        let signature = self
        .eth_signer
        .generate_signature(signing_hash.as_ref()).unwrap();

        signature
    }
}


// pub fn adjust_field_names(serialized: &str) -> Result<String, regex::Error> {
//     let re = Regex::new(r#""price_x18":"#)?; // Match the exact field name in quotes
//     Ok(re.replace_all(serialized, r#""priceX18":"#).into_owned()) // Replace with the desired field name
// }

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