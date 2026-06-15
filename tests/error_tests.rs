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

#[test]
fn test_downcast_and_leaf() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = NemesisError::new("Origin", io_err)
        .add_ctx("ctx")
        .add_source("Athena");

    // std::error::Error::source should return the payload source
    use std::error::Error;
    assert!(err.source().is_some());

    // leaf_error should traverse to the root cause
    let leaf = err.leaf_error();
    assert_eq!(leaf.to_string(), "file not found");

    // downcast_ref should downcast to std::io::Error
    let io_downcast = err.downcast_ref::<io::Error>();
    assert!(io_downcast.is_some());
    assert_eq!(io_downcast.unwrap().kind(), io::ErrorKind::NotFound);
}

#[test]
fn test_walk_chain() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = NemesisError::new("Origin", io_err)
        .add_ctx("context origin")
        .add_source("Athena")
        .add_ctx("context athena")
        .add_source("Talos");

    let mut iter = err.walk_chain();
    let step1 = iter.next().unwrap();
    assert_eq!(step1.source_name(), "Talos");
    
    let step2 = iter.next().unwrap();
    assert_eq!(step2.source_name(), "Athena");
    
    let step3 = iter.next().unwrap();
    assert_eq!(step3.source_name(), "Origin");
    
    assert!(iter.next().is_none());
}

#[test]
fn test_display_format() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = NemesisError::new("Origin", io_err)
        .add_ctx("Unable to open file: config.xff")
        .add_source("Athena")
        .add_ctx("Loading config file during startup");

    let formatted = format!("{}", err);
    let expected = "Error: file not found\n\
                      Context: Loading config file during startup\n\
                      Source: Athena\n\
                    \x20\x20\x20\x20Error: file not found\n\
                    \x20\x20\x20\x20\x20\x20Context: Unable to open file: config.xff\n\
                    \x20\x20\x20\x20\x20\x20Source: Origin\n";
    assert_eq!(formatted, expected);
}
