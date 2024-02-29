use serde_json::json;
use tonic::{Request, Response, Status};
use log::{info, error};

use crate::{
   connectors::vertex::payload_signer::Signer, domain::models::vertex::sol_structs::Order, services::vertex::helper::VertexHelper, shared::utils::type_conv::pad_to_fixed_bytes32, vertex_execute::{
        vertex_execute_service_server::VertexExecuteService, CancelAllForProductRequest, CancelAndPlaceRequest, CancelOrderRequest, CancelOrderResponse, PlaceOrderRequest, PlaceOrderResponse
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
        // Ensure the order is present in the request
        let order_request = place_order_request.order.ok_or_else(|| Status::invalid_argument("Order is missing in the request"))?;


        
        // Construct the Order struct from the request to Order Request from alloy Sol
        let order = Order {
            sender: pad_to_fixed_bytes32(&order_request.sender).map_err(|e| Status::internal(format!("Sender conversion error: {}", e)))?,
            priceX18: order_request.price_x18.parse().map_err(|e| Status::internal(format!("Price parsing error: {}", e)))?,
            amount: order_request.amount.parse().map_err(|e| Status::internal(format!("Amount parsing error: {}", e)))?,
            expiration: self.generate_expiration_time(3600),
            nonce: self.generate_nonce(),
        };

        // Use the Signer to construct and sign the order payload
        let signer = Signer::new();
        let signature = signer.sign_place_order_payload(&order);
    
        // signer.sign_subscription_auth_payload(sender_address)
        let place_order_payload = json!({
            "place_order": {
                "product_id": place_order_request.product_id,
                "order": {
                    "sender": order_request.sender, // Assuming sender is a String
                    "price_x18": &order.priceX18.to_string(),
                    "amount": &order.amount.to_string(),
                    "expiration": &order.expiration,
                    "nonce": &order.nonce
                },
                "signature": signature,
                "id": place_order_request.id,
            }
        }).to_string();

        // println!("payload {:?}",order_payload);
        match self.gateway_client.send_message(place_order_payload).await {
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
                    "sender": cancel_order_request.sender, 
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
        let cancel_order_request = request.into_inner();
        let cancel_all_payload = json!({
            "cancel_product_orders": {
                "tx": {
                    "sender": cancel_order_request.sender,
                    "productIds": cancel_order_request.product_ids,
                    "nonce": self.generate_nonce()
                },
                "signature": "", // Generate signature
                "digest": null
            }
        });

        let payload_str = cancel_all_payload.to_string();
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

    async fn cancel_and_place(&self, request: Request<CancelAndPlaceRequest>) -> Result<Response<PlaceOrderResponse>, Status> {
        // Destructure `request` to get both `cancel_order_request` and `place_order_request` in one step.
        let inner = request.into_inner();
        let cancel_order_request = inner.cancel_order_request.unwrap();
        let place_order_request = inner.place_order_request.unwrap();
    
        let cancel_and_place_payload = json!({
            "cancel_and_place": {
                "cancel_tx": {
                    "sender":  cancel_order_request.sender,
                    "productIds": cancel_order_request.product_ids,
                    "digests": cancel_order_request.digests,
                    "nonce": self.generate_nonce()
                },
                "cancel_signature": "", // Generate cancel signature
                "place_order": place_order_request
            }
        });
    
        let payload_str = cancel_and_place_payload.to_string();
        match self.gateway_client.send_message(payload_str).await {
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
}

