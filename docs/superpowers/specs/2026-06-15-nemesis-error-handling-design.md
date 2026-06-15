# Design Spec: Nemesis Error Handling

## Metadata
- **Status:** Approved
- **Date:** 2026-06-15
- **Project:** Nemesis (The error handling crate for the Pantheon ecosystem)

## Goals
1. **Context Preservation:** Provide `.add_ctx()` to attach contextual information as errors bubble up crate boundaries.
2. **Crate Boundary Isolation:** Provide `.add_source()` to wrap error layers under new boundaries (e.g., Athena, Talos) to clearly trace the path of propagation.
3. **Programmatic Interaction:** Expose an iterator to walk the nested error chain, locate the root cause leaf error, and downcast to a concrete error type (e.g., `std::io::Error`).
4. **Pretty Printing:** Implement a recursive display format with nested 4-space indentations and 2-space detail offsets for clear, structured debugging outputs.
5. **Error Collections:** Offer `NemesisCollection` to compile multiple parallel errors (useful during startup/initialization phases).
6. **Zero Dependencies:** Rely solely on the Rust standard library.

## Data Structures

### 1. `NemesisPayload`
Represents the error payload at any given layer.
```rust
#[derive(Debug)]
pub enum NemesisPayload {
    /// A leaf error wrapping a standard or third-party error
    Leaf(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A nested error wrapping an upstream NemesisError
    Nested(Box<NemesisError>),
}
```

### 2. `NemesisError`
The core error container representing a nested layer in the error stack.
```rust
#[derive(Debug)]
pub struct NemesisError {
    /// Crate or boundary that added this layer of context (e.g. "Athena", "Talos")
    source: &'static str,
    /// Stack of contextual messages added at this layer
    context: Vec<String>,
    /// The actual error payload at this layer (either a leaf error or a nested NemesisError)
    payload: NemesisPayload,
}
```

### 3. `NemesisChainIter`
An iterator walking the nested error layers from the outermost layer to the innermost leaf.
```rust
pub struct NemesisChainIter<'a> {
    current: Option<&'a NemesisError>,
}
```

### 4. `NemesisCollection`
A first-class container for accumulating multiple errors.
```rust
#[derive(Debug)]
pub struct NemesisCollection {
    name: String,
    errors: Vec<NemesisError>,
}
```

## Programmatic API and Implementation Details

### `NemesisError` Methods
- `new<E>(source: &'static str, err: E) -> Self`: Constructs a root leaf error layer.
- `add_ctx(mut self, ctx: impl Into<String>) -> Self`: Appends context to the current layer.
- `add_source(self, source: &'static str) -> Self`: Wraps the current error into a new nested `NemesisError` layer under the given source.
- `source_name(&self) -> &'static str`: Returns the static source name.
- `contexts(&self) -> &[String]`: Returns a slice of context messages.
- `walk_chain(&self) -> NemesisChainIter<'_>`: Returns the chain iterator.
- `payload(&self) -> &NemesisPayload`: Accesses the internal payload.
- `leaf_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static)`: Traverses to the innermost leaf error.
- `downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T>`: Helper to downcast the leaf error.

### `std::error::Error` Implementations
- `NemesisError` implements `std::error::Error`, returning its payload (`Leaf`'s inner error or `Nested`'s inner `NemesisError`) in the `source()` method.
- `NemesisCollection` implements `std::error::Error`.

### Extension Trait: `NemesisResultExt`
Helper trait implemented for `Result<T, E> where E: Into<NemesisError>` to allow fluent nesting:
- `.add_ctx(ctx: impl Into<String>) -> Result<T, NemesisError>`
- `.add_source(source: &'static str) -> Result<T, NemesisError>`

## Formatting Requirements

### `NemesisError` display output structure:
```text
Error: <innermost leaf error message>
  Context: <layer context 1>
  Context: <layer context 2>
  Source: <layer source>
    Error: <innermost leaf error message>
      Context: <nested layer context>
      Source: <nested layer source>
```
Nesting rules:
- Incremental indentation level: `4` spaces.
- Local offset for Context/Source lines: `2` spaces.

### `NemesisCollection` display output structure:
```text
Error collection '<name>':
  Error: <error 1 message>
    Context: ...
    Source: ...
  Error: <error 2 message>
    ...
```
Collection elements must be indented by `2` spaces initially, and format recursively.
