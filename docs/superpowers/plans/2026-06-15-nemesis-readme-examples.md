# Nemesis README and Examples Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a comprehensive runnable example to the `examples` directory and update `README.md` to document usage patterns.

**Architecture:** Create a new example file under the `examples` directory and integrate its usage in `README.md`.

**Tech Stack:** Rust (standard library only).

---

### Task 1: Create Runnable Example

**Files:**
- Create: `examples/usage.rs`

- [ ] **Step 1: Write the example code**
  Create [examples/usage.rs](file:///home/xqhare/Adytum/Programming/rust/nemesis/examples/usage.rs) illustrating all major features of the crate (layer nesting, context adding, display output, chain iteration, downcasting, and collection):
  ```rust
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
  ```

- [ ] **Step 2: Run the example**
  Run: `cargo run --example usage`
  Expected: The example compiles, executes, and outputs the single error formatting, chain walking, downcasting information, and the collection formatting correctly.

- [ ] **Step 3: Commit**
  ```bash
  git add examples/usage.rs
  git commit -m "feat(examples): add runnable usage example illustrating all core APIs"
  ```

---

### Task 2: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write updated usage section**
  Modify [README.md](file:///home/xqhare/Adytum/Programming/rust/nemesis/README.md) to document the API and display formats.
  ```markdown
  ## Usage

  ### Importing

  Add the following to your `Cargo.toml`:

  ```toml
  [dependencies]
  nemesis = { git = "https://github.com/xqhare/nemesis" }
  ```

  ### Example Usage

  Here is a simple example showing how to nest errors, add contexts, print them, walk the error layers, and downcast to the leaf error:

  ```rust
  use std::io;
  use nemesis::{NemesisError, NemesisResultExt};

  fn read_config(path: &str) -> Result<String, NemesisError> {
      std::fs::read_to_string(path).map_err(|err| {
          NemesisError::new("Origin", err).add_ctx(format!("Failed to read file: {path}"))
      })
  }

  fn load_config() -> Result<String, NemesisError> {
      read_config("config.xff")
          .add_source("Athena")
          .add_ctx("Loading subsystem config during startup")
  }

  fn main() {
      if let Err(err) = load_config() {
          // Print formatted nested error hierarchy
          eprintln!("{}", err);

          // Programmatic check: downcast to leaf standard error
          if let Some(io_err) = err.downcast_ref::<io::Error>() {
              if io_err.kind() == io::ErrorKind::NotFound {
                  eprintln!("Configuration file not found.");
              }
          }
      }
  }
  ```

  For a complete overview of all APIs (including walking error chains and using `NemesisCollection`), see the runnable example under [examples/usage.rs](examples/usage.rs).
  ```

- [ ] **Step 2: Verify formatting and correctness**
  Verify the Markdown formatting and spelling in `README.md`.

- [ ] **Step 3: Commit**
  ```bash
  git add README.md
  git commit -m "docs(readme): add usage documentation and links to examples"
  ```
