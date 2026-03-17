use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Encode error: {0}")]
    Encode(String),
    #[error("Decode error: {0}")]
    Decode(String),
}

/// MessagePack codec for RPC messages.
/// Encodes/decodes Map<String, Value> to/from msgpack bytes.
/// Equivalent to Java's MessagePackCodec.
pub struct MessagePackCodec;

impl MessagePackCodec {
    /// Encode a map to msgpack bytes.
    pub fn encode(map: &HashMap<String, rmpv::Value>) -> Result<Vec<u8>, CodecError> {
        let value = rmpv::Value::Map(
            map.iter()
                .map(|(k, v)| (rmpv::Value::String(k.clone().into()), v.clone()))
                .collect(),
        );
        rmp_serde::to_vec(&value).map_err(|e| CodecError::Encode(e.to_string()))
    }

    /// Decode msgpack bytes to a map.
    pub fn decode(data: &[u8]) -> Result<HashMap<String, rmpv::Value>, CodecError> {
        let value: rmpv::Value =
            rmp_serde::from_slice(data).map_err(|e| CodecError::Decode(e.to_string()))?;

        match value {
            rmpv::Value::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match k {
                        rmpv::Value::String(s) => s.into_str().unwrap_or_default().to_string(),
                        _ => k.to_string(),
                    };
                    map.insert(key, v);
                }
                Ok(map)
            }
            _ => Err(CodecError::Decode("Expected map at top level".into())),
        }
    }
}

/// Null-safe helper for extracting typed values from RPC response maps.
/// Equivalent to Java's MapHelper.
pub struct MapHelper;

impl MapHelper {
    pub fn get_int(map: &HashMap<String, rmpv::Value>, key: &str) -> i64 {
        map.get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    pub fn get_float(map: &HashMap<String, rmpv::Value>, key: &str) -> f64 {
        map.get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn get_string(map: &HashMap<String, rmpv::Value>, key: &str) -> String {
        map.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn get_bool(map: &HashMap<String, rmpv::Value>, key: &str) -> bool {
        map.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn get_map(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> HashMap<String, rmpv::Value> {
        map.get(key)
            .and_then(|v| {
                if let rmpv::Value::Map(pairs) = v {
                    let mut m = HashMap::new();
                    for (k, v) in pairs {
                        let key = match k {
                            rmpv::Value::String(s) => {
                                s.as_str().unwrap_or_default().to_string()
                            }
                            _ => k.to_string(),
                        };
                        m.insert(key, v.clone());
                    }
                    Some(m)
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn get_list(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> Vec<rmpv::Value> {
        map.get(key)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_map_list(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> Vec<HashMap<String, rmpv::Value>> {
        Self::get_list(map, key)
            .into_iter()
            .filter_map(|v| {
                if let rmpv::Value::Map(pairs) = v {
                    let mut m = HashMap::new();
                    for (k, v) in pairs {
                        let key = match k {
                            rmpv::Value::String(s) => {
                                s.as_str().unwrap_or_default().to_string()
                            }
                            _ => k.to_string(),
                        };
                        m.insert(key, v);
                    }
                    Some(m)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_int_list(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> Vec<i64> {
        Self::get_list(map, key)
            .into_iter()
            .filter_map(|v| v.as_i64())
            .collect()
    }

    pub fn get_string_list(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> Vec<String> {
        Self::get_list(map, key)
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    }

    pub fn get_bytes(
        map: &HashMap<String, rmpv::Value>,
        key: &str,
    ) -> Vec<u8> {
        map.get(key)
            .and_then(|v| {
                if let rmpv::Value::Binary(b) = v {
                    Some(b.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}
