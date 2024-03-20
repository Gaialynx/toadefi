use alloy_primitives::FixedBytes;
use log::{error, info};
use serde_json::json;
use tonic::{Request, Response, Status};

use crate::{
    connectors::vertex::payload_signer::Signer,
    domain::models::vertex::sol_structs::Order,
    services::vertex::helper::VertexHelper,
    shared::utils::type_conv::{self, fixed_bytes_to_hex, vec_to_fixed_bytes32},
    vertex_execute::{
        vertex_execute_service_server::VertexExecuteService, CancelAllForProductRequest,
        CancelAndPlaceRequest, CancelOrderRequest, CancelOrderResponse, PlaceOrderRequest,
        PlaceOrderResponse,
    },
};

use super::client::VertexClient;

#[tonic::async_trait]
impl VertexExecuteService for VertexClient {
    // places order on vertex
    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        let place_order_request = request.into_inner();
        let order_request = place_order_request
            .order
            .ok_or_else(|| Status::invalid_argument("Order is missing in the request"))?;

        const DEFAULT_SUBACCOUNT: &str = "64656661756c740000000000";
        let sender_full_hex = format!(
            "{:0<64}",
            format!(
                "{}{}",
                &order_request.sender,
                DEFAULT_SUBACCOUNT // default subaccount - replace when we create our own subacc
            )
        );

        // Multiply with 1e18 to real life pricing standard. Src: vertex doc
        let price_x18 = type_conv::string_and_i128(&order_request.price_x18);
        let amount_x18 = type_conv::string_and_i128(&order_request.amount);

        let clean_sender_hex = sender_full_hex.trim_start_matches("0x");
        let address_bytes = type_conv::hex_to_bytes(&clean_sender_hex);
        let expiration_time = self.generate_expiration_time(1000, 0);

        // Construct the Order struct from the request to Order Request from alloy Sol
        let order = Order {
            sender: vec_to_fixed_bytes32(address_bytes).unwrap(),
            priceX18: price_x18,
            amount: amount_x18,
            expiration: expiration_time,
            nonce: self.generate_nonce(),
        };

        let ordr_addrs: Option<String> =
            self.get_contract_addr(place_order_request.product_id).await;

        // With Verifying Contract
        let signer = Signer::new(ordr_addrs);
        let signature = signer.sign_place_order_payload(&order);

        let payload = json!({
            "place_order": {
                "product_id": place_order_request.product_id,
                "order": {
                    "sender": sender_full_hex, // Assuming sender is a String
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

        match self.gateway_client.send_message(payload).await {
            Ok(response_data) => match serde_json::from_str::<PlaceOrderResponse>(&response_data) {
                Ok(response) => Ok(Response::new(response)),
                Err(e) => Err(Status::internal(format!(
                    "Failed to parse gateway response: {}",
                    e
                ))),
            },
            Err(e) => Err(Status::internal(format!(
                "Failed to send order to gateway: {}",
                e
            ))),
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

        // let cancel_order = Cancellation {
        //     sender: pad_to_fixed_bytes32(&cancel_order_request.sender)
        //         .map_err(|e| Status::internal(format!("Sender conversion error: {}", e)))?,
        //     productIds: cancel_order_request.product_ids.clone(),
        //     digests: digests_fixed.clone(),
        //     nonce: self.generate_nonce(),
        // };

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
