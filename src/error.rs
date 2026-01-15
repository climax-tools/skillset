use thiserror::Error;

pub type Result<T> = std::result::Result<T, SkillsetError>;

#[derive(Error, Debug)]
pub enum SkillsetError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Source error: {0}")]
    Source(String),

    #[error("Convention error: {0}")]
    Convention(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("OCI error: {0}")]
    Oci(String),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Convention not found: {0}")]
    ConventionNotFound(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Invalid skill name: {0}")]
    InvalidSkillName(String),
}
