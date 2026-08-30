//! Unit tests for SerializationUtils CBOR helpers.

#[cfg(test)]
mod tests {
    #[test]
    fn test_serialize_i32() {
        #[derive(serde::Serialize)]
        struct TestVal {
            v: i32,
        }
        let val = TestVal { v: 42 };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_serialize_string() {
        #[derive(serde::Serialize)]
        struct TestStr {
            s: String,
        }
        let val = TestStr { s: "hello".into() };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_serialize_vec() {
        #[derive(serde::Serialize)]
        struct TestVec {
            v: Vec<i32>,
        }
        let val = TestVec { v: vec![1, 2, 3] };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_serialize_option() {
        #[derive(serde::Serialize)]
        struct TestOpt {
            v: Option<u32>,
        }
        let val = TestOpt { v: Some(42) };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_serialize_nested() {
        #[derive(serde::Serialize)]
        struct Inner {
            x: i32,
        }
        #[derive(serde::Serialize)]
        struct Outer {
            inner: Inner,
        }
        let val = Outer {
            inner: Inner { x: 7 },
        };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_to_cbor_empty_vec() {
        #[derive(serde::Serialize)]
        struct Empty {
            v: Vec<i32>,
        }
        let val = Empty { v: vec![] };
        let bytes = cearaafis::transparency::to_cbor(&val).expect("should serialize");
        assert!(!bytes.is_empty());
    }
}
