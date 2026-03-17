use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    Serialize(String),
    #[error("Deserialization error: {0}")]
    Deserialize(String),
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
}

/// An RPC request sent to the game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub params: Vec<rmpv::Value>,
}

/// An RPC response from the game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<rmpv::Value>,
    pub error: Option<RpcError>,
}

/// An RPC error returned by the game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Encode an RPC request into a length-prefixed msgpack frame.
pub fn encode_request(request: &RpcRequest) -> Result<Vec<u8>, ProtocolError> {
    let payload = rmp_serde::to_vec(request)
        .map_err(|e| ProtocolError::Serialize(e.to_string()))?;

    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decode a length-prefixed msgpack frame into an RPC response.
pub fn decode_response(data: &[u8]) -> Result<RpcResponse, ProtocolError> {
    if data.len() < 4 {
        return Err(ProtocolError::InvalidFrame("Frame too short".into()));
    }

    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len {
        return Err(ProtocolError::InvalidFrame(format!(
            "Frame declares {} bytes but only {} available",
            len,
            data.len() - 4
        )));
    }

    let payload = &data[4..4 + len];
    rmp_serde::from_slice(payload)
        .map_err(|e| ProtocolError::Deserialize(e.to_string()))
}
