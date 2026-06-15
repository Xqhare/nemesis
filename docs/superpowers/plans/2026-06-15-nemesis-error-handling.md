# Nemesis Error Handling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Nemesis error-handling library to support context preservation, crate boundary isolation, error chain iteration, error collections, and structured formatting.

**Architecture:** Use a consolidated module architecture where the core types (`NemesisError`, `NemesisPayload`, `NemesisChainIter`, `NemesisCollection`, and `NemesisResultExt`) are implemented in `src/error.rs` and re-exported in `src/lib.rs`.

**Tech Stack:** Rust standard library (zero external dependencies).

---

### Task 1: Basic Structures and Creation

**Files:**
- Create: `src/error.rs`
- Test: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify `NemesisError::new` can construct a leaf error.
  Create `tests/error_tests.rs`:
  ```rust
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
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (types not defined/implemented).

- [ ] **Step 3: Write minimal implementation**
  Create/overwrite `src/error.rs` with the basic structs:
  ```rust
  use std::fmt;

  #[derive(Debug)]
  pub enum NemesisPayload {
      Leaf(Box<dyn std::error::Error + Send + Sync + 'static>),
      Nested(Box<NemesisError>),
  }

  #[derive(Debug)]
  pub struct NemesisError {
      source: &'static str,
      context: Vec<String>,
      payload: NemesisPayload,
  }

  impl NemesisError {
      pub fn new<E>(source: &'static str, err: E) -> Self
      where
          E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
      {
          Self {
              source,
              context: Vec::new(),
              payload: NemesisPayload::Leaf(err.into()),
          }
      }

      pub fn source_name(&self) -> &'static str {
          self.source
      }

      pub fn contexts(&self) -> &[String] {
          &self.context
      }

      pub fn payload(&self) -> &NemesisPayload {
          &self.payload
      }
  }

  impl fmt::Display for NemesisError {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          write!(f, "NemesisError from {}", self.source)
      }
  }

  impl std::error::Error for NemesisError {}
  ```
  Expose the module and types in `src/lib.rs`:
  ```rust
  mod error;
  pub use error::{NemesisError, NemesisPayload};
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs src/lib.rs tests/error_tests.rs
  git commit -m "add(error): basic NemesisError struct and creation"
  ```

---

### Task 2: Context Preservation & Boundary Isolation

**Files:**
- Modify: `src/error.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test in `tests/error_tests.rs` to verify `.add_ctx()` and `.add_source()` builder methods.
  ```rust
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
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (methods `add_ctx` and `add_source` not found).

- [ ] **Step 3: Write minimal implementation**
  Add `add_ctx` and `add_source` in `src/error.rs`:
  ```rust
  impl NemesisError {
      // Existing methods ...

      pub fn add_ctx(mut self, ctx: impl Into<String>) -> Self {
          self.context.push(ctx.into());
          self
      }

      pub fn add_source(self, source: &'static str) -> Self {
          Self {
              source,
              context: Vec::new(),
              payload: NemesisPayload::Nested(Box::new(self)),
          }
      }
  }
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs tests/error_tests.rs
  git commit -m "add(error): context and source nesting builders"
  ```

---

### Task 3: Error Trait Implementation & Downcasting

**Files:**
- Modify: `src/error.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify `std::error::Error::source`, `leaf_error`, and `downcast_ref` work correctly.
  ```rust
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
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (methods `leaf_error` and `downcast_ref` not found).

- [ ] **Step 3: Write minimal implementation**
  Update `std::error::Error::source` and implement `leaf_error` and `downcast_ref` in `src/error.rs`:
  ```rust
  impl std::error::Error for NemesisError {
      fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
          match &self.payload {
              NemesisPayload::Leaf(err) => Some(err.as_ref()),
              NemesisPayload::Nested(cause) => Some(cause.as_ref()),
          }
      }
  }

  impl NemesisError {
      // Existing methods ...

      pub fn leaf_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
          let mut current = self;
          while let NemesisPayload::Nested(ref cause) = current.payload {
              current = cause;
          }
          match &current.payload {
              NemesisPayload::Leaf(err) => err.as_ref(),
              NemesisPayload::Nested(_) => unreachable!(),
          }
      }

      pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
          self.leaf_error().downcast_ref::<T>()
      }
  }
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs tests/error_tests.rs
  git commit -m "add(error): leaf_error and downcast_ref support"
  ```

---

### Task 4: Programmatic Iteration

**Files:**
- Modify: `src/error.rs`
- Modify: `src/lib.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify walking the error chain using `walk_chain`.
  ```rust
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
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (`NemesisChainIter` and `walk_chain` not found).

- [ ] **Step 3: Write minimal implementation**
  Add `NemesisChainIter` and implement `walk_chain` in `src/error.rs`:
  ```rust
  pub struct NemesisChainIter<'a> {
      current: Option<&'a NemesisError>,
  }

  impl<'a> Iterator for NemesisChainIter<'a> {
      type Item = &'a NemesisError;

      fn next(&mut self) -> Option<Self::Item> {
          let prev = self.current;
          if let Some(err) = prev {
              self.current = match &err.payload {
                  NemesisPayload::Nested(cause) => Some(cause.as_ref()),
                  NemesisPayload::Leaf(_) => None,
              };
          }
          prev
      }
  }

  impl NemesisError {
      // Existing methods ...

      pub fn walk_chain(&self) -> NemesisChainIter<'_> {
          NemesisChainIter { current: Some(self) }
      }
  }
  ```
  Expose `NemesisChainIter` in `src/lib.rs`:
  ```rust
  // Modify src/lib.rs to re-export NemesisChainIter
  mod error;
  pub use error::{NemesisError, NemesisPayload, NemesisChainIter};
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs src/lib.rs tests/error_tests.rs
  git commit -m "add(error): chain iterator for error nesting"
  ```

---

### Task 5: Formatting Specification

**Files:**
- Modify: `src/error.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify the display output format of a nested error.
  ```rust
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
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: FAIL (formatted output does not match expected output).

- [ ] **Step 3: Write minimal implementation**
  Update `fmt::Display` and implement `format_with_indent` in `src/error.rs`:
  ```rust
  impl fmt::Display for NemesisError {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          self.format_with_indent(f, 0)
      }
  }

  impl NemesisError {
      // Existing methods ...

      pub fn format_with_indent(&self, f: &mut fmt::Formatter<'_>, base_indent: usize) -> fmt::Result {
          let pad = " ".repeat(base_indent);
          let detail_pad = " ".repeat(base_indent + 2);

          let error_msg = match &self.payload {
              NemesisPayload::Nested(_) => self.leaf_error().to_string(),
              NemesisPayload::Leaf(err) => err.to_string(),
          };

          writeln!(f, "{}Error: {}", pad, error_msg)?;
          for ctx in &self.context {
              writeln!(f, "{}Context: {}", detail_pad, ctx)?;
          }
          writeln!(f, "{}Source: {}", detail_pad, self.source)?;

          if let NemesisPayload::Nested(ref cause) = self.payload {
              cause.format_with_indent(f, base_indent + 4)?;
          }

          Ok(())
      }
  }
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs tests/error_tests.rs
  git commit -m "add(error): Display format implementation"
  ```

---

### Task 6: Error Collections

**Files:**
- Modify: `src/error.rs`
- Modify: `src/lib.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify `NemesisCollection` functions and its display format.
  ```rust
  use nemesis::NemesisCollection;

  #[test]
  fn test_error_collection() {
      let mut collection = NemesisCollection::new("startup");
      assert!(collection.is_empty());
      assert_eq!(collection.len(), 0);
      assert!(collection.into_result().is_ok());

      let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
      let err1 = NemesisError::new("Athena", io_err);
      
      let format_err = io::Error::new(io::ErrorKind::InvalidData, "invalid data");
      let err2 = NemesisError::new("Talos", format_err);

      let mut collection = NemesisCollection::new("startup");
      collection.push(err1);
      collection.push(err2);

      assert!(!collection.is_empty());
      assert_eq!(collection.len(), 2);

      let mut iter = collection.iter();
      assert_eq!(iter.next().unwrap().source_name(), "Athena");
      assert_eq!(iter.next().unwrap().source_name(), "Talos");
      assert!(iter.next().is_none());

      let formatted = format!("{}", collection);
      let expected = "Error collection 'startup':\n\
                      \x20\x20Error: file not found\n\
                      \x20\x20\x20\x20Source: Athena\n\
                      \x20\x20Error: invalid data\n\
                      \x20\x20\x20\x20Source: Talos\n";
      assert_eq!(formatted, expected);

      assert!(collection.into_result().is_err());
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (`NemesisCollection` not found).

- [ ] **Step 3: Write minimal implementation**
  Add `NemesisCollection` structure and its implementations in `src/error.rs`:
  ```rust
  #[derive(Debug)]
  pub struct NemesisCollection {
      name: String,
      errors: Vec<NemesisError>,
  }

  impl NemesisCollection {
      pub fn new(name: impl Into<String>) -> Self {
          Self {
              name: name.into(),
              errors: Vec::new(),
          }
      }

      pub fn push(&mut self, err: impl Into<NemesisError>) {
          self.errors.push(err.into());
      }

      pub fn is_empty(&self) -> bool {
          self.errors.is_empty()
      }

      pub fn len(&self) -> usize {
          self.errors.len()
      }

      pub fn into_result(self) -> Result<(), Self> {
          if self.errors.is_empty() {
              Ok(())
          } else {
              Err(self)
          }
      }

      pub fn iter(&self) -> std::slice::Iter<'_, NemesisError> {
          self.errors.iter()
      }
  }

  impl std::error::Error for NemesisCollection {}

  impl fmt::Display for NemesisCollection {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          writeln!(f, "Error collection '{}':", self.name)?;
          for err in &self.errors {
              err.format_with_indent(f, 2)?;
          }
          Ok(())
      }
  }
  ```
  Expose `NemesisCollection` in `src/lib.rs`:
  ```rust
  // Modify src/lib.rs
  mod error;
  pub use error::{NemesisError, NemesisPayload, NemesisChainIter, NemesisCollection};
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs src/lib.rs tests/error_tests.rs
  git commit -m "add(error): NemesisCollection implementation"
  ```

---

### Task 7: Extension Trait

**Files:**
- Modify: `src/error.rs`
- Modify: `src/lib.rs`
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the failing test**
  Add a test to verify the fluent `.add_ctx()` and `.add_source()` methods on a Result type using `NemesisResultExt`.
  ```rust
  use nemesis::NemesisResultExt;

  #[test]
  fn test_result_ext() {
      let res: Result<(), NemesisError> = Err(NemesisError::new("Origin", io::Error::new(io::ErrorKind::Other, "err")));
      let annotated = res.add_ctx("context string").add_source("Athena");

      assert!(annotated.is_err());
      let err = annotated.unwrap_err();
      assert_eq!(err.source_name(), "Athena");
      
      let mut iter = err.walk_chain();
      let step1 = iter.next().unwrap();
      assert!(step1.contexts().is_empty());
      
      let step2 = iter.next().unwrap();
      assert_eq!(step2.source_name(), "Origin");
      assert_eq!(step2.contexts(), &["context string".to_string()]);
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test error_tests`
  Expected: Compile failure (`NemesisResultExt` not found or not implemented).

- [ ] **Step 3: Write minimal implementation**
  Add `NemesisResultExt` to `src/error.rs`:
  ```rust
  pub trait NemesisResultExt<T, E> {
      fn add_ctx(self, ctx: impl Into<String>) -> Result<T, NemesisError>;
      fn add_source(self, source: &'static str) -> Result<T, NemesisError>;
  }

  impl<T, E> NemesisResultExt<T, E> for Result<T, E>
  where
      E: Into<NemesisError>,
  {
      fn add_ctx(self, ctx: impl Into<String>) -> Result<T, NemesisError> {
          self.map_err(|e| e.into().add_ctx(ctx))
      }

      fn add_source(self, source: &'static str) -> Result<T, NemesisError> {
          self.map_err(|e| e.into().add_source(source))
      }
  }
  ```
  Expose `NemesisResultExt` in `src/lib.rs`:
  ```rust
  // Modify src/lib.rs
  mod error;
  pub use error::{NemesisError, NemesisPayload, NemesisChainIter, NemesisCollection, NemesisResultExt};
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test error_tests`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add src/error.rs src/lib.rs tests/error_tests.rs
  git commit -m "add(error): NemesisResultExt extension trait"
  ```

---

### Task 8: Cleanup and Final Verification

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Clean up `lib.rs`**
  Remove the placeholder `add` function and its tests in `src/lib.rs`. Ensure only the necessary header and the re-exports are present.
  ```rust
  #![doc = include_str!("../README.md")]
  #![warn(missing_docs)]
  #![warn(clippy::pedantic)]
  #![warn(clippy::all)]
  #![warn(clippy::restriction)]
  #![expect(
      clippy::missing_docs_in_private_items,
      clippy::print_stdout,
      clippy::implicit_return,
      clippy::single_call_fn,
      clippy::str_to_string,
      clippy::question_mark_used,
      clippy::indexing_slicing,
      clippy::pattern_type_mismatch,
      clippy::arbitrary_source_item_ordering,
      clippy::doc_paragraphs_missing_punctuation,
      clippy::exhaustive_enums,
      clippy::min_ident_chars,
      clippy::missing_trait_methods,
      clippy::impl_trait_in_params,
      clippy::as_conversions,
      clippy::cast_lossless,
      clippy::shadow_reuse,
      clippy::blanket_clippy_restriction_lints,
      clippy::doc_include_without_cfg,
      reason = "Ignored warnings"
  )]

  mod error;
  pub use error::{NemesisChainIter, NemesisCollection, NemesisError, NemesisPayload, NemesisResultExt};
  ```

- [ ] **Step 2: Run verification commands**
  Run:
  - `cargo check`
  - `cargo test`
  - `cargo clippy --all-targets`
  - `cargo fmt -- --check`
  Expected: All checks and tests pass cleanly without warning or format errors.

- [ ] **Step 3: Commit**
  ```bash
  git add src/lib.rs
  git commit -m "cleanup(lib): remove boilerplate, clean up re-exports"
  ```
