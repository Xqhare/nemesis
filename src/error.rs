use std::fmt;

/// The payload of a [`NemesisError`], representing either a root leaf error or a nested error.
///
/// This enum allows constructing error chains, where a high-level error context wraps a
/// nested cause, eventually leading to a leaf (root) error.
#[derive(Debug)]
pub enum NemesisPayload {
    /// A leaf error. This wraps the root cause of the error chain, implementing
    /// `std::error::Error`, `Send`, `Sync`, and having a static lifetime.
    Leaf(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A nested error. This wraps another [`NemesisError`], representing a step
    /// in the propagation and wrapping of errors.
    Nested(Box<NemesisError>),
}

/// A structured error representation in the Nemesis ecosystem.
///
/// `NemesisError` captures:
/// - A static string slice denoting the source (e.g., file location, component, or system name).
/// - An accumulated list of context strings.
/// - A payload that is either a boxed leaf error or a nested `NemesisError`.
///
/// It supports building nested context chains, tracing down to the leaf error,
/// and downcasting to custom error types.
#[derive(Debug)]
pub struct NemesisError {
    source: &'static str,
    context: Vec<String>,
    payload: NemesisPayload,
}

impl NemesisError {
    /// Creates a new `NemesisError` representing a leaf error with the specified source location.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    /// let err = NemesisError::new("db::read", io_err);
    /// assert_eq!(err.source_name(), "db::read");
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

    /// Returns the static source name or location label associated with this error level.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("parser", "invalid token");
    /// assert_eq!(err.source_name(), "parser");
    /// ```
    pub fn source_name(&self) -> &'static str {
        self.source
    }

    /// Returns a slice of all context strings added at this specific error level.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("parser", "invalid token")
    ///     .add_ctx("failed to parse config file");
    /// assert_eq!(err.contexts(), &["failed to parse config file".to_string()]);
    /// ```
    pub fn contexts(&self) -> &[String] {
        &self.context
    }

    /// Returns a reference to the underlying [`NemesisPayload`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisError, NemesisPayload};
    ///
    /// let err = NemesisError::new("engine", "engine failure");
    /// match err.payload() {
    ///     NemesisPayload::Leaf(_) => println!("It is a leaf error"),
    ///     NemesisPayload::Nested(_) => unreachable!(),
    /// }
    /// ```
    pub fn payload(&self) -> &NemesisPayload {
        &self.payload
    }

    /// Adds a context string to the current error level and returns the modified error.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("network", "connection reset")
    ///     .add_ctx("failed fetching user profile")
    ///     .add_ctx("retry attempt 3");
    /// assert_eq!(err.contexts().len(), 2);
    /// ```
    pub fn add_ctx(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    /// Wraps the current error as a nested error with a new static source name.
    ///
    /// This is useful when bubbling an error up to a higher-level module or component.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("filesystem", "permission denied")
    ///     .add_source("app::load_config");
    /// assert_eq!(err.source_name(), "app::load_config");
    /// ```
    pub fn add_source(self, source: &'static str) -> Self {
        Self {
            source,
            context: Vec::new(),
            payload: NemesisPayload::Nested(Box::new(self)),
        }
    }

    /// Traverses the entire nested error chain to retrieve a reference to the root leaf error.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("filesystem", "permission denied")
    ///     .add_source("app::load_config");
    /// let leaf = err.leaf_error();
    /// assert_eq!(leaf.to_string(), "permission denied");
    /// ```
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

    /// Attempts to downcast the root leaf error to a concrete reference of type `T`.
    ///
    /// Returns `Some(&T)` if the downcast succeeds, or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::AddrInUse, "port taken");
    /// let err = NemesisError::new("server", io_err).add_source("app::run");
    ///
    /// if let Some(real_io_err) = err.downcast_ref::<io::Error>() {
    ///     assert_eq!(real_io_err.kind(), io::ErrorKind::AddrInUse);
    /// } else {
    ///     panic!("failed to downcast");
    /// }
    /// ```
    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        self.leaf_error().downcast_ref::<T>()
    }

    /// Returns an iterator to walk the chain of nested `NemesisError`s.
    ///
    /// The iteration starts with the outermost error and moves inward to the root cause.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisError;
    ///
    /// let err = NemesisError::new("filesystem", "permission denied")
    ///     .add_source("app::load_config");
    ///
    /// let sources: Vec<&str> = err.walk_chain().map(|e| e.source_name()).collect();
    /// assert_eq!(sources, vec!["app::load_config", "filesystem"]);
    /// ```
    pub fn walk_chain(&self) -> NemesisChainIter<'_> {
        NemesisChainIter {
            current: Some(self),
        }
    }

    /// Formats this error hierarchy with a specified base indentation level.
    ///
    /// This method recursively formats nested errors, writing details like source, contexts, and messages.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if writing to the formatter fails.
    pub fn format_with_indent(
        &self,
        f: &mut fmt::Formatter<'_>,
        base_indent: usize,
    ) -> fmt::Result {
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

/// An iterator over the chain of nested [`NemesisError`]s.
///
/// Created by the [`NemesisError::walk_chain`] method.
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

impl fmt::Display for NemesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_with_indent(f, 0)
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

/// A collection of [`NemesisError`]s, used to aggregate multiple failures.
///
/// This is particularly useful in validation or batch processing scenarios where you want
/// to report all failures rather than stopping at the first one.
#[derive(Debug)]
pub struct NemesisCollection {
    name: String,
    errors: Vec<NemesisError>,
}

impl NemesisCollection {
    /// Creates a new `NemesisCollection` with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisCollection;
    ///
    /// let collection = NemesisCollection::new("config validation");
    /// assert!(collection.is_empty());
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            errors: Vec::new(),
        }
    }

    /// Pushes a new error into the collection.
    ///
    /// The parameter `err` can be any type that implements `Into<NemesisError>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisCollection, NemesisError};
    ///
    /// let mut collection = NemesisCollection::new("batch job");
    /// collection.push(NemesisError::new("job_1", "timeout"));
    /// assert_eq!(collection.len(), 1);
    /// ```
    pub fn push(&mut self, err: impl Into<NemesisError>) {
        self.errors.push(err.into());
    }

    /// Returns `true` if the collection contains no errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::NemesisCollection;
    ///
    /// let collection = NemesisCollection::new("checks");
    /// assert!(collection.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of errors currently in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisCollection, NemesisError};
    ///
    /// let mut collection = NemesisCollection::new("checks");
    /// assert_eq!(collection.len(), 0);
    /// collection.push(NemesisError::new("check_1", "failed"));
    /// assert_eq!(collection.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Converts the collection into a `Result`.
    ///
    /// Returns `Ok(())` if the collection is empty, or `Err(self)` if it contains errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisCollection, NemesisError};
    ///
    /// let collection = NemesisCollection::new("empty checks");
    /// assert!(collection.into_result().is_ok());
    ///
    /// let mut collection = NemesisCollection::new("failed checks");
    /// collection.push(NemesisError::new("check_1", "failed"));
    /// assert!(collection.into_result().is_err());
    /// ```
    pub fn into_result(self) -> Result<(), Self> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    /// Returns an iterator over the reference of all [`NemesisError`]s in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisCollection, NemesisError};
    ///
    /// let mut collection = NemesisCollection::new("validation");
    /// collection.push(NemesisError::new("field_1", "invalid format"));
    ///
    /// for err in collection.iter() {
    ///     println!("Error in source: {}", err.source_name());
    /// }
    /// ```
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

/// An extension trait for [`Result`] to easily wrap or annotate errors with Nemesis context and sources.
pub trait NemesisResultExt<T, E> {
    /// Maps the error of a `Result` to a [`NemesisError`] and appends a context string.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisError, NemesisResultExt};
    ///
    /// let res: Result<(), NemesisError> = Err(NemesisError::new("network", "connection reset"));
    /// // Since E: Into<NemesisError>, we can add context using `add_ctx`.
    /// let res_with_ctx = res.add_ctx("retrying database query");
    /// assert!(res_with_ctx.is_err());
    /// ```
    fn add_ctx(self, ctx: impl Into<String>) -> Result<T, NemesisError>;

    /// Maps the error of a `Result` to a [`NemesisError`] and wraps it with a new source label.
    ///
    /// # Examples
    ///
    /// ```
    /// use nemesis::{NemesisError, NemesisResultExt};
    ///
    /// let res: Result<(), NemesisError> = Err(NemesisError::new("network", "connection reset"));
    /// let res_with_source = res.add_source("app::initialize");
    /// assert_eq!(res_with_source.unwrap_err().source_name(), "app::initialize");
    /// ```
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
