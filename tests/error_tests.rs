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
