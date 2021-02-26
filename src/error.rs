use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("There was an error parsing your request: {reason:?}")]
    ParseError { reason: String },
    #[error("Failed to bind to address")]
    ConnectionError,
    #[error("Got an invalid request")]
    InvalidRequest,
    #[error("Received no request from client")]
    NoRequestFound,
    #[error("Failed to load response")]
    NoResponseFound,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Request was improperly formatted: {code:?}")]
    InvalidRequest { code: u32 },
    #[error("No key found in request")]
    MissingKey,
}
