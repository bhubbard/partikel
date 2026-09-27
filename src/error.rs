use thiserror::Error;

#[derive(Error, Debug)]
pub enum PartikelError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Catalog error: {0}")]
    Catalog(String),

    #[error("Catalog error: {0}")]
    CatalogError(String),

    #[error("Tier 0 error: {0}")]
    Tier0(String),

    #[error("MCP protocol error: {0}")]
    Mcp(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, PartikelError>;
