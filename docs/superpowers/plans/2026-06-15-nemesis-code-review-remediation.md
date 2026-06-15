# Nemesis Code Review Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remediate all issues, bugs, and compiler/clippy warnings identified during the code review of the `nemesis` crate.

**Architecture:** Maintain the existing zero-dependency module structure where all implementations reside in `src/error.rs` and are re-exported in `src/lib.rs`. Fix test spacing, add full public API documentation, and remove unused lint expectations.

**Tech Stack:** Rust (standard library only).

---

### Task 1: Fix Test Formatting Bug

**Files:**
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Write the updated test code**
  Modify [tests/error_tests.rs](file:///home/xqhare/Adytum/Programming/rust/nemesis/tests/error_tests.rs) to include explicit space escapes (`\x20`) for the first layer's details to prevent the line continuation backslash (`\`) from stripping leading spaces.
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
                      \x20\x20Context: Loading config file during startup\n\
                      \x20\x20Source: Athena\n\
                      \x20\x20\x20\x20Error: file not found\n\
                      \x20\x20\x20\x20\x20\x20Context: Unable to open file: config.xff\n\
                      \x20\x20\x20\x20\x20\x20Source: Origin\n";
      assert_eq!(formatted, expected);
  }
  ```

- [ ] **Step 2: Run tests to verify the suite passes**
  Run: `cargo test --test error_tests`
  Expected: All 7 tests pass successfully (test result: ok).

- [ ] **Step 3: Commit**
  ```bash
  git add tests/error_tests.rs
  git commit -m "fix(tests): resolve formatting space stripping in test_display_format"
  ```

---

### Task 2: Add Public API Documentation

**Files:**
- Modify: `src/error.rs`

- [ ] **Step 1: Write documentation comments**
  Add full `///` markdown documentation comments and examples to all public structs, enums, variants, traits, and methods in [src/error.rs](file:///home/xqhare/Adytum/Programming/rust/nemesis/src/error.rs).
  ```rust
  use std::fmt;

  /// The payload of a `NemesisError` representing either a root leaf error or a nested upstream error.
  #[derive(Debug)]
  pub enum NemesisPayload {
      /// A leaf error wrapping a standard or third-party error.
      Leaf(Box<dyn std::error::Error + Send + Sync + 'static>),
      /// A nested error wrapping an upstream `NemesisError`.
      Nested(Box<NemesisError>),
  }

  /// The core error container representing a nested error layer in the stack.
  #[derive(Debug)]
  pub struct NemesisError {
      source: &'static str,
      context: Vec<String>,
      payload: NemesisPayload,
  }

  impl NemesisError {
      /// Constructs a new root leaf error layer.
      ///
      /// # Examples
      /// ```
      /// use std::io;
      /// use nemesis::NemesisError;
      ///
      /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
      /// let err = NemesisError::new("Origin", io_err);
      /// assert_eq!(err.source_name(), "Origin");
      /// ```
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

      /// Returns the static source name of this error layer.
      pub fn source_name(&self) -> &'static str {
          self.source
      }

      /// Returns a slice of context messages attached to this error layer.
      pub fn contexts(&self) -> &[String] {
          &self.context
      }

      /// Returns a reference to the error payload at this layer.
      pub fn payload(&self) -> &NemesisPayload {
          &self.payload
      }

      /// Appends a context message to the current error layer.
      pub fn add_ctx(mut self, ctx: impl Into<String>) -> Self {
          self.context.push(ctx.into());
          self
      }

      /// Wraps the current error into a new nested `NemesisError` layer under the given source.
      pub fn add_source(self, source: &'static str) -> Self {
          Self {
              source,
              context: Vec::new(),
              payload: NemesisPayload::Nested(Box::new(self)),
          }
      }

      /// Traverses the error chain to locate and return the innermost leaf error.
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

      /// Attempts to downcast the innermost leaf error to a specific concrete type.
      pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
          self.leaf_error().downcast_ref::<T>()
      }

      /// Returns an iterator walking the nested error layers from the outermost to the innermost leaf.
      pub fn walk_chain(&self) -> NemesisChainIter<'_> {
          NemesisChainIter { current: Some(self) }
      }

      /// Recursively formats the error with structured indentation.
      ///
      /// # Errors
      /// Returns a `fmt::Error` if writing to the formatter fails.
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

  /// An iterator walking the nested error layers from the outermost layer to the innermost leaf.
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

  /// A first-class container for accumulating multiple errors (e.g. during startup phase).
  #[derive(Debug)]
  pub struct NemesisCollection {
      name: String,
      errors: Vec<NemesisError>,
  }

  impl NemesisCollection {
      /// Constructs a new, empty error collection.
      pub fn new(name: impl Into<String>) -> Self {
          Self {
              name: name.into(),
              errors: Vec::new(),
          }
      }

      /// Pushes an error into the collection.
      pub fn push(&mut self, err: impl Into<NemesisError>) {
          self.errors.push(err.into());
      }

      /// Returns `true` if no errors have been collected.
      pub fn is_empty(&self) -> bool {
          self.errors.is_empty()
      }

      /// Returns the number of collected errors.
      pub fn len(&self) -> usize {
          self.errors.len()
      }

      /// Converts the collection into a `Result` (`Ok` if empty, `Err` if it contains errors).
      ///
      /// # Errors
      /// Returns `Err(Self)` containing the accumulated errors if not empty.
      pub fn into_result(self) -> Result<(), Self> {
          if self.errors.is_empty() {
              Ok(())
          } else {
              Err(self)
          }
      }

      /// Returns an iterator over references to the collected errors.
      pub fn iter(&self) -> std::slice::Iter<'_, NemesisError> {
          self.errors.iter()
      }
  }

  /// An extension trait implemented for `Result` to allow fluent nesting and contextualization.
  pub trait NemesisResultExt<T, E> {
      /// Attaches a context message to the error.
      ///
      /// # Errors
      /// Returns a `NemesisError` containing the added context message if self is `Err`.
      fn add_ctx(self, ctx: impl Into<String>) -> Result<T, NemesisError>;

      /// Bubbles up the error and marks a new boundary source.
      ///
      /// # Errors
      /// Returns a `NemesisError` containing the new source boundary if self is `Err`.
      fn add_source(self, source: &'static str) -> Result<T, NemesisError>;
  }
  ```

- [ ] **Step 2: Run cargo check to verify compilation**
  Run: `cargo check`
  Expected: Successful compilation without warnings in library code.

- [ ] **Step 3: Commit**
  ```bash
  git add src/error.rs
  git commit -m "docs(error): add comprehensive API documentation comments"
  ```

---

### Task 3: Clean Up Clippy Lint Expectations

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Remove unfulfilled expectations**
  In [src/lib.rs](file:///home/xqhare/Adytum/Programming/rust/nemesis/src/lib.rs), clean up the `#![expect(...)]` block to only contain expectations that are active, or convert them to `#![allow(...)]` to avoid unfulfilled lint expectations compiler warnings.
  Specifically, change `expect` to `allow`:
  ```rust
  #![allow(
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
      clippy::doc_include_without_cfg
  )]
  ```

- [ ] **Step 2: Run cargo clippy to verify**
  Run: `cargo clippy --lib`
  Expected: Successful lint check with zero warnings or expectation panics.

- [ ] **Step 3: Commit**
  ```bash
  git add src/lib.rs
  git commit -m "refactor(lib): change strict clippy expectations to allows to prevent unfulfilled warnings"
  ```

---

### Task 4: Fix Test Warnings

**Files:**
- Modify: `tests/error_tests.rs`

- [ ] **Step 1: Clean up unused `mut` and use `io::Error::other`**
  Modify [tests/error_tests.rs](file:///home/xqhare/Adytum/Programming/rust/nemesis/tests/error_tests.rs):
  1. Remove duplicate and unused mut `let mut collection` on line 99:
     ```rust
     let collection = NemesisCollection::new("startup");
     ```
  2. Use `io::Error::other` in `test_result_ext` on line 137:
     ```rust
     let res: Result<(), NemesisError> = Err(NemesisError::new("Origin", io::Error::other("err")));
     ```

- [ ] **Step 2: Run verification checks**
  Run:
  - `cargo test`
  - `cargo clippy --all-targets`
  - `cargo fmt -- --check`
  Expected: Complete test execution, zero clippy warnings, and correct formatting check.

- [ ] **Step 3: Commit**
  ```bash
  git add tests/error_tests.rs
  git commit -m "fix(tests): resolve clippy warnings for unused mutability and deprecated constructors"
  ```
