
I want a unified error type that can be used for the entire pantheon.

## Problems to solve

The way I currently handle errors looses a lot of context. I often just bubble it up directly, making it hard to debug.

I also cast many errors into new types so I can use `?` to handle them (e.g. casting an `AthenaError` into a `TalosError`. The resulting error doesnt have any additional context meaning that the error output can be: `IO error, unable to open file`. What file? Why do I want to open it? Config, logging?? Is it an error raised by talos or upstream somewhere?)

## What I want

Most important is programmatically interacting with the error.
Second is easy understanding of the printed output in `stderr`.

Ideally an error should present like this:

```bash
$ cargo run
$ Error: IO error.
$   Context: Loading config file during startup.
$   Source: Athena // <-- Optional
$       Error: IO error
$           Context: Unable to open file: path/to/file.xff\n\n{...} 
$           Source: Origin // Maybe do this if this is the source of the error? Would make source no longer optional, but a lot more obvious.
```

Meaning I need a `.add_ctx` method and a `.add_source` method.
Also an implementation of `Display`, maybe even `Debug`.

The above really is just a nested error.

### Idea

Also have a type for a vector of errors, to be able to collect errors together.

```bash
$ Error collection 'startup':
$   Error: IO error.
$       Description: Loading config file during startup.
$       Context: Passed file 'path/to/file.xff' to athena to load for startup config loading.
$       Source: Athena
$           Error: IO error
$               Context: Unable to open file: path/to/file.xff\n\n{...} 
$               Source: Origin
$   Error: Parsing error.
$       Description: Unable to parse 'a_string' into a valid 'usize'.
$       Context: Tried to populate field 'a_field' inside config struct during startup.
$       Source: This_library
```

## Solution

Instead of doing my knee jerk reaction of trying to build a generalised struct, I think defining a trait or type would be better.
Really just define the API for the error type.


# Refined Design

## Goals

1. **Context Preservation:** Allow attaching context at boundary levels (`.add_ctx()`) to avoid losing debugging details as errors bubble up.
2. **Crate Boundary Isolation:** Allow wrapping errors in a new outer layer representing the boundary source (`.add_source()`) to clearly trace the path of the error.
3. **Programmatic Interaction:** Expose clean APIs to walk the error chain, extract context, and downcast to the innermost leaf error.
4. **Pretty Printing:** Format errors with clean, predictable, nested indentation levels in `stderr`.
5. **Error Collections:** Provide a first-class type to accumulate multiple errors during startup or validation phases.
6. **Zero Dependencies:** Built entirely on the Rust standard library.

---

## Core Architecture

### 1. `NemesisError` Struct
The core container representing a single nested error layer:

```rust
use std::fmt;

#[derive(Debug)]
pub enum NemesisPayload {
    /// A leaf error wrapping a standard or third-party error
    Leaf(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A nested error wrapping an upstream NemesisError
    Nested(Box<NemesisError>),
}

#[derive(Debug)]
pub struct NemesisError {
    /// Crate or boundary that added this layer of context (e.g. "Athena", "Talos")
    source: &'static str,
    /// Stack of contextual messages added at this layer
    context: Vec<String>,
    /// The actual error payload at this layer (either a leaf error or a nested NemesisError)
    payload: NemesisPayload,
}

impl NemesisError {
    /// Create a new root error from any standard error type
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

    /// Add a context string to the current error layer
    pub fn add_ctx(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    /// Wrap the current error in a new outer error layer with a different source
    pub fn add_source(self, source: &'static str) -> Self {
        Self {
            source,
            context: Vec::new(),
            payload: NemesisPayload::Nested(Box::new(self)),
        }
    }
}

impl std::error::Error for NemesisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.payload {
            NemesisPayload::Leaf(err) => Some(err.as_ref()),
            NemesisPayload::Nested(cause) => Some(cause.as_ref()),
        }
    }
}

```

---

## Programmatic API

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
    /// Returns the source name for this specific error layer (e.g. "Athena")
    pub fn source_name(&self) -> &'static str {
        self.source
    }

    /// Returns the list of context strings attached at this layer
    pub fn contexts(&self) -> &[String] {
        &self.context
    }

    /// Walk the NemesisError chain from the outermost layer down to the innermost layer
    pub fn walk_chain(&self) -> NemesisChainIter<'_> {
        NemesisChainIter { current: Some(self) }
    }

    /// Access the payload of this error layer
    pub fn payload(&self) -> &NemesisPayload {
        &self.payload
    }

    /// Locate the leaf (innermost) error in the chain, which represents the root cause
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

    /// Try to downcast the leaf error to a specific concrete type
    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        self.leaf_error().downcast_ref::<T>()
    }
}
```

---

## Error Collections

For collecting multiple parallel errors (e.g., during initialization/validation phases):

```rust
#[derive(Debug)]
pub struct NemesisCollection {
    name: String,
    errors: Vec<NemesisError>,
}

impl NemesisCollection {
    /// Create a new, empty error collection
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            errors: Vec::new(),
        }
    }

    /// Push an error into the collection
    pub fn push(&mut self, err: impl Into<NemesisError>) {
        self.errors.push(err.into());
    }

    /// Returns true if no errors have been collected
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of collected errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Converts the collection into a Result (Ok if empty, Err if contains errors)
    pub fn into_result(self) -> Result<(), Self> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    /// Iterate over references of the collected errors
    pub fn iter(&self) -> std::slice::Iter<'_, NemesisError> {
        self.errors.iter()
    }
}

impl std::error::Error for NemesisCollection {}
```

---

## Extension Trait

Allows inline annotation and conversion:

```rust
pub trait NemesisResultExt<T, E> {
    /// Attach context to the error
    fn add_ctx(self, ctx: impl Into<String>) -> Result<T, NemesisError>;
    
    /// Bubble up the error and mark a new source boundary
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

---

## Formatting Specification

### Standard Display Format

The `Display` implementations must recursively pad strings using a 4-space nesting increment and a 2-space offset for details (Context and Source):

```rust
impl fmt::Display for NemesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_with_indent(f, 0)
    }
}

impl NemesisError {
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

---

## Example Usage

### 1. Root Level (e.g., in Athena)
```rust
use std::fs::File;
use Nemesis::{NemesisError, NemesisResultExt};

pub fn read_config(path: &str) -> Result<File, NemesisError> {
    File::open(path).map_err(|e| {
        NemesisError::new("Origin", e).add_ctx(format!("Unable to open file: {path}"))
    })
}
```

### 2. Upstream Propagation (e.g., in Talos wrapping Athena)
```rust
pub fn load_athena_config() -> Result<(), NemesisError> {
    read_config("path/to/file.xff")
        .add_source("Athena")
        .add_ctx("Loading config file during startup")?;
    Ok(())
}
```

### 3. Application Main Handling
```rust
fn main() {
    if let Err(err) = load_athena_config() {
        eprintln!("{}", err);
        
        // Programmatic check: check if root error was std::io::Error
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::NotFound {
                // Handle missing file
            }
        }
    }
}
```
