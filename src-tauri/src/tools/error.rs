use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("path escapes the target directory")]
    PathEscape,
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("not a valid UTF-8 text file")]
    NotUtf8,
    #[error("invalid search pattern: {0}")]
    InvalidPattern(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for ToolError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
