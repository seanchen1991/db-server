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
    #[error("Failed to load persisted data")]
    LoadError,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Request was improperly formatted: {code:?}")]
    InvalidRequest { code: u32 },
    #[error("No key found in request")]
    MissingKey,
}
