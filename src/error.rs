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

impl fmt::Display for NemesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NemesisError from {}", self.source)
    }
}

impl std::error::Error for NemesisError {}
