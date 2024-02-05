use alloy_sol_macro::sol;
use serde::Serialize;

sol! {
    struct StreamAuthentication {
        bytes32 sender;
        uint64 expiration;
    }
}

#[derive(Serialize)]
pub struct SerializableStreamAuthentication {
    sender: String,
    expiration: String,
}

impl From<&StreamAuthentication> for SerializableStreamAuthentication {
    fn from(auth: &StreamAuthentication) -> Self {
        let sender = hex::encode(auth.sender.as_slice());
        let expiration = auth.expiration.to_string();

        SerializableStreamAuthentication { sender, expiration }
    }
}
