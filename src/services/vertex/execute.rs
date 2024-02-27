use alloy_sol_types::Eip712Domain;
use serde_json::json;
use tonic::{Request, Response, Status};
use log::{info, error};

use crate::{
    config::CONFIG, connectors::vertex::payload_signer::Signer, domain::models::vertex::sol_structs::Order, services::vertex::helper::VertexHelper, shared::utils::{eth_signer::EthSigner, type_conv::{hex_to_fixed_bytes32, pad_to_fixed_bytes32, vec_to_fixed_bytes32}}, vertex_execute::{
        vertex_execute_service_server::VertexExecuteService, CancelAllForProductRequest, CancelOrderRequest, CancelOrderResponse, PlaceOrderRequest, PlaceOrderResponse
    }
};

use super::client::VertexClient;

#[tonic::async_trait]
impl VertexExecuteService for VertexClient {
    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        let place_order_request = request.into_inner();
        let order_request = match place_order_request.order {
            Some(order) => order,
            None => return Err(Status::invalid_argument("Order is missing in the request")),
        };


        
        // Construct the Order struct from the request to Order Request from alloy Sol
        let order = Order {
            sender: match pad_to_fixed_bytes32(order_request.sender) {
                Ok(sender) => sender,
                Err(e) => return Err(Status::internal(format!("Sender conversion error: {}", e))),
            },
            priceX18: match order_request.price_x18.parse() {
                Ok(price) => price,
                Err(e) => return Err(Status::internal(format!("Price parsing error: {}", e))),
            },
            amount: match order_request.amount.parse() {
                Ok(amount) => amount,
                Err(e) => return Err(Status::internal(format!("Amount parsing error: {}", e))),
            },
            expiration: self.generate_expiration_time(3600),
            nonce: self.generate_nonce(),
        };

        // Create an EthSigner instance and bind it to a variable
        let eth_signer = EthSigner::new(&CONFIG.private_key);
        let eip712_domain = Eip712Domain::default();

        // Use the Signer to construct and sign the order payload
        let signer = Signer::new(&CONFIG.sender_address, &eth_signer, &eip712_domain);

        let order_payload = match signer.construct_order_payload(&order, place_order_request.product_id.parse().unwrap(), place_order_request.id.unwrap() as i32) {
            Ok(payload) => payload,
            Err(e) => {
                error!("Failed to construct order payload: {}", e);
                // Handle the error here, potentially returning an error or logging more information
                return Err(Status::internal("Failed to construct order payload"));
            },
        };

        println!("payload {:?}",order_payload);
        match self.gateway_client.send_message(order_payload).await {
            Ok(response_data) => {
                // Log the raw response data for debugging
                info!("Raw gateway response: {}", response_data);
        
                match serde_json::from_str::<PlaceOrderResponse>(&response_data) {
                    Ok(response) => {
                        info!("Order placed successfully");
                        Ok(Response::new(response))
                    },
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        Err(Status::internal("Failed to parse gateway response"))
                    },
                }
            },
            Err(e) => {
                error!("Failed to send order to gateway: {}", e);
                Err(Status::internal("Failed to send order to gateway"))
            },
        }
    }

    async fn cancel_order(&self, request: Request<CancelOrderRequest>,) -> Result<Response<CancelOrderResponse>, Status> {
        let cancel_order_request = request.into_inner();
        let cancel_payload = json!({
            "cancel_orders": {
                "tx": {
                    "sender": cancel_order_request.sender, // Assuming you have a sender_address field in VertexClient
                    "productIds": cancel_order_request.product_ids,
                    "digests": cancel_order_request.digests,
                    "nonce": self.generate_nonce()
                },
                "signature": "" // You'll need to generate a signature based on your protocol
            }
        });

        let payload_str = cancel_payload.to_string();
        match self.gateway_client.send_message(payload_str).await {
            Ok(response_data) => {
                // Log the raw response data for debugging
                info!("Raw gateway response: {}", response_data);
        
                match serde_json::from_str::<CancelOrderResponse>(&response_data) {
                    Ok(response) => {
                        info!("Order placed successfully");
                        Ok(Response::new(response))
                    },
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        Err(Status::internal("Failed to parse gateway response"))
                    },
                }
            },
            Err(e) => {
                error!("Failed to send order to gateway: {}", e);
                Err(Status::internal("Failed to send order to gateway"))
            },
        }
    }

    async fn cancel_all_for_product(&self, request: Request<CancelAllForProductRequest>) -> Result<Response<CancelOrderResponse>, Status> {
        let cancel_all_payload = json!({
            "cancel_product_orders": {
                "tx": {
                    "sender": self.sender_address,
                    "productIds": [product_id],
                    "nonce": self.generate_nonce()
                },
                "signature": "", // Generate signature
                "digest": null
            }
        });

        let payload_str = cancel_all_payload.to_string();
        self.send_message_to_gateway(payload_str).await.map_err(|e| Status::internal(e.to_string()))
    }

    async fn cancel_and_place(&self, cancel_product_ids: Vec<u32>, cancel_digests: Vec<String>, new_order: Order) -> Result<(), Status> {
        // Assuming you have a method to generate the payload for a new order
        let new_order_payload = self.generate_order_payload(&new_order)?;

        let cancel_and_place_payload = json!({
            "cancel_and_place": {
                "cancel_tx": {
                    "sender": self.sender_address,
                    "productIds": cancel_product_ids,
                    "digests": cancel_digests,
                    "nonce": self.generate_nonce()
                },
                "cancel_signature": "", // Generate cancel signature
                "place_order": new_order_payload
            }
        });

        let payload_str = cancel_and_place_payload.to_string();
        self.send_message_to_gateway(payload_str).await.map_err(|e| Status::internal(e.to_string()))
    }
}

