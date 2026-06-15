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

    pub fn walk_chain(&self) -> NemesisChainIter<'_> {
        NemesisChainIter { current: Some(self) }
    }

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
