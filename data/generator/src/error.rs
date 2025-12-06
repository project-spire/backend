#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    #[error("Namespace collision: {0}")]
    NamespaceCollision(String),

    #[error("Unknown type: {0}")]
    UnknownType(String),

    #[error("Collect error: {0}")]
    Collect(String),

    #[error("Inheritance error: {0} -> {1}")]
    Inheritance(String, String),
    
    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),
}
