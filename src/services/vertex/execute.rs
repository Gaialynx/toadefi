use alloy_primitives::FixedBytes;
use log::{error, info};
use serde_json::json;
use tonic::{Request, Response, Status};

use crate::{
    connectors::vertex::payload_signer::Signer,
    domain::models::vertex::sol_structs::{Cancellation, Order},
    services::vertex::helper::VertexHelper,
    shared::utils::type_conv::{fixed_bytes_to_hex, pad_to_fixed_bytes32, vec_to_fixed_bytes32},
    vertex_execute::{
        vertex_execute_service_server::VertexExecuteService, CancelAllForProductRequest,
        CancelAndPlaceRequest, CancelOrderRequest, CancelOrderResponse, PlaceOrderRequest,
        PlaceOrderResponse,
    },
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
        let order_request = place_order_request
            .order
            .ok_or_else(|| Status::invalid_argument("Order is missing in the request"))?;

        let expiration_time = self.generate_expiration_time(1000, 1);
        let padded_sender = pad_to_fixed_bytes32(&order_request.sender);
        // Construct the Order struct from the request to Order Request from alloy Sol
        let order = Order {
            sender: padded_sender
                .map_err(|e| Status::internal(format!("Sender conversion error: {}", e)))?,
            priceX18: order_request
                .price_x18
                .parse()
                .map_err(|e| Status::internal(format!("Price parsing error: {}", e)))?,
            amount: order_request
                .amount
                .parse()
                .map_err(|e| Status::internal(format!("Amount parsing error: {}", e)))?,
            expiration: expiration_time,
            nonce: self.generate_nonce(),
        };

        // Use the Signer to construct and sign the order payload
        let signer = Signer::new();
        let signature = signer.sign_place_order_payload(&order);

        let paddedSender = pad_to_fixed_bytes32(&order_request.sender)
            .unwrap()
            .to_string();
        // signer.sign_subscription_auth_payload(sender_address)
        let place_order_payload = json!({
            "place_order": {
                "product_id": place_order_request.product_id,
                "order": {
                    "sender": paddedSender, // Assuming sender is a String
                    "priceX18": &order.priceX18.to_string(),
                    "amount": &order.amount.to_string(),
                    "expiration": &order.expiration.to_string(),
                    "nonce": &order.nonce.to_string()
                },
                "signature": signature,
                "id": place_order_request.id,
            }
        })
        .to_string();

        println!("{}", place_order_payload.clone());

        // println!("payload {:?}",order_payload);
        match self
            .gateway_client
            .send_message(place_order_payload.clone())
            .await
        {
            Ok(response_data) => {
                info!("Raw gateway response: {}", response_data);

                match serde_json::from_str::<PlaceOrderResponse>(&response_data) {
                    Ok(response) => {
                        info!("Order placed successfully.");
                        Ok(Response::new(response))
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse gateway response: {}. Response data: '{}'",
                            e, response_data
                        );
                        Err(Status::internal(format!(
                            "Failed to parse gateway response. Error: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to send order to gateway. Error: {}, Payload: '{}'",
                    e,
                    place_order_payload.clone()
                );
                Err(Status::internal(format!(
                    "Failed to send order to gateway. Error: {}",
                    e
                )))
            }
        }
    }

    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
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
                    }
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        Err(Status::internal("Failed to parse gateway response"))
                    }
                }
            }
            Err(e) => {
                error!("Failed to send order to gateway: {}", e);
                Err(Status::internal("Failed to send order to gateway"))
            }
        }
    }

    async fn cancel_all_for_product(
        &self,
        request: Request<CancelAllForProductRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {
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
                    }
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        Err(Status::internal("Failed to parse gateway response"))
                    }
                }
            }
            Err(e) => {
                error!("Failed to send order to gateway: {}", e);
                Err(Status::internal("Failed to send order to gateway"))
            }
        }
    }

    async fn cancel_and_place(
        &self,
        request: Request<CancelAndPlaceRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        let inner = request.into_inner();
        let cancel_order_request = inner
            .cancel_order_request
            .ok_or_else(|| Status::invalid_argument("Order is missing in the request"))?;
        let place_order_request = inner.place_order_request.unwrap();

        let digests_fixed: Result<Vec<FixedBytes<32>>, _> = cancel_order_request
            .digests
            .into_iter()
            .map(|digest| vec_to_fixed_bytes32(digest)) // Use the conversion utility function
            .collect(); // Collect into a Result<Vec<FixedBytes<32>>, Error>

        // Check if the conversion was successful
        let digests_fixed = digests_fixed
            .map_err(|e| Status::internal(format!("Error converting digests: {}", e)))?;

        let cancel_order = Cancellation {
            sender: pad_to_fixed_bytes32(&cancel_order_request.sender)
                .map_err(|e| Status::internal(format!("Sender conversion error: {}", e)))?,
            productIds: cancel_order_request.product_ids.clone(),
            digests: digests_fixed.clone(),
            nonce: self.generate_nonce(),
        };

        let digests_hex: Vec<String> = digests_fixed
            .iter()
            .map(|fixed_bytes| fixed_bytes_to_hex(fixed_bytes))
            .collect();

        let cancel_and_place_payload = json!({
            "cancel_and_place": {
                "cancel_tx": {
                    "sender": cancel_order_request.sender.clone(), // Assuming this is already a string
                    "productIds": cancel_order_request.product_ids.clone(),
                    "digests": digests_hex, // Use the hex string vector
                    "nonce": self.generate_nonce()
                },
                "cancel_signature": "", // Placeholder for cancel signature
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
                    }
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        Err(Status::internal("Failed to parse gateway response"))
                    }
                }
            }
            Err(e) => {
                error!("Failed to send order to gateway: {}", e);
                Err(Status::internal("Failed to send order to gateway"))
            }
        }
    }
}
