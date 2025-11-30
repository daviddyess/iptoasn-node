use thiserror::Error;

/**
 * All possible errors in the application
 */
#[derive(Error, Debug)]
pub enum AppError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),

    #[error("Failed to parse response: {0}")]
    HttpParse(String),

    #[error("Database parse error: {0}")]
    DatabaseParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    #[error("Database not loaded")]
    DatabaseNotLoaded,
}

/**
 * Result type alias
 */
pub type Result<T> = std::result::Result<T, AppError>;
