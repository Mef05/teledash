use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Disconnected source")]
    Disconnected,
}
