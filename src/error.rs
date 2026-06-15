use std::fmt;

pub type NemesisResult<T> = Result<T, NemesisError>;

#[derive(Debug)]
pub enum NemesisError {
    Generic(String),
    Io(std::io::Error),
}

impl fmt::Display for NemesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NemesisError::Generic(msg) => write!(f, "{}", msg),
            NemesisError::Io(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for NemesisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NemesisError::Generic(_) => None,
            NemesisError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for NemesisError {
    fn from(err: std::io::Error) -> Self {
        NemesisError::Io(err)
    }
}
