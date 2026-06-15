use std::io;
use nemesis::{NemesisError, NemesisPayload};

#[test]
fn test_new_leaf_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = NemesisError::new("Origin", io_err);
    assert_eq!(err.source_name(), "Origin");
    assert!(err.contexts().is_empty());
    match err.payload() {
        NemesisPayload::Leaf(_) => {},
        _ => panic!("Expected Leaf payload"),
    }
}

#[test]
fn test_add_ctx_and_source() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let err = NemesisError::new("Origin", io_err)
        .add_ctx("context 1")
        .add_ctx("context 2");

    assert_eq!(err.contexts(), &["context 1".to_string(), "context 2".to_string()]);

    let nested_err = err.add_source("Athena");
    assert_eq!(nested_err.source_name(), "Athena");
    assert!(nested_err.contexts().is_empty());
    match nested_err.payload() {
        NemesisPayload::Nested(_) => {},
        _ => panic!("Expected Nested payload"),
    }
}
