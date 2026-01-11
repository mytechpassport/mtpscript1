use crate::json::serialize::Json;

pub fn encode_cbor(json: &Json) -> Vec<u8> {
    // Basic CBOR encoding; use a crate like ciborium in real impl
    vec![0xA0] // Placeholder
}
