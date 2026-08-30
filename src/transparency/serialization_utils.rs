/// SerializationUtils: CBOR serialization helpers — mirrors .NET SerializationUtils.cs.
use serde::ser::Serialize;

/// Serialize any serializable data to CBOR bytes.
pub fn serialize<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
    serde_cbor::to_vec(data).map_err(|e| e.to_string())
}

/// Serialize directly to bytes (convenient single-call version).
pub fn to_cbor<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
    serde_cbor::to_vec(data).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple() {
        #[derive(serde::Serialize)]
        struct TestStruct {
            value: i32,
        }

        let obj = TestStruct { value: 42 };
        let bytes = serialize(&obj).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_to_cbor() {
        #[derive(serde::Serialize)]
        struct TestStruct {
            value: i32,
        }

        let obj = TestStruct { value: 42 };
        let bytes = to_cbor(&obj).expect("should serialize");
        assert!(!bytes.is_empty());
    }
}
