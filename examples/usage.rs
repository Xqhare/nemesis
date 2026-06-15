use std::io;
use nemesis::{NemesisCollection, NemesisError, NemesisResultExt};

fn read_config(path: &str) -> Result<String, NemesisError> {
    std::fs::read_to_string(path).map_err(|err| {
        NemesisError::new("Origin", err).add_ctx(format!("Failed to read file: {path}"))
    })
}

fn load_subsystem_config() -> Result<String, NemesisError> {
    read_config("nonexistent_config.xff")
        .add_source("Athena")
        .add_ctx("Loading config for the Athena subsystem during startup")
}

fn run_app() -> Result<(), NemesisError> {
    load_subsystem_config()
        .add_source("Talos")
        .add_ctx("Initializing Talos app engine")?;
    Ok(())
}

fn main() {
    println!("--- Single Error Chain formatting ---");
    if let Err(err) = run_app() {
        eprintln!("{err}");

        println!("--- Programmatic downcasting check ---");
        if let Some(io_err) = err.downcast_ref::<io::Error>() {
            println!("Root cause is an IO error of kind: {:?}", io_err.kind());
        }

        println!("\n--- Walking the error chain ---");
        for (i, layer) in err.walk_chain().enumerate() {
            println!(
                "Layer {}: Source = '{}', Contexts = {:?}",
                i,
                layer.source_name(),
                layer.contexts()
            );
        }
    }

    println!("\n--- Error Collection example ---");
    let mut collection = NemesisCollection::new("startup_validation");

    let parse_err = io::Error::new(io::ErrorKind::InvalidInput, "Failed to parse port number");
    collection.push(NemesisError::new("ConfigParser", parse_err).add_ctx("Port field validation failed"));

    let connection_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Database offline");
    collection.push(NemesisError::new("DatabaseConnector", connection_err).add_ctx("Failed to ping database"));

    if let Err(coll) = collection.into_result() {
        eprintln!("{coll}");
    }
}
