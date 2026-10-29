use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Network API error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Math error: {0}")]
    MathError(String),

    #[error("Could not calculate stats for empty matrix")]
    EmptyMatrix,
}

pub type Result<T> = std::result::Result<T, GatewayError>;
