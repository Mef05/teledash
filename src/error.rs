use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Disconnected source")]
    Disconnected,
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Failed to convert JSON into SensorPreset: {0}")]
    ConvertError(#[from] serde_json::Error),

    #[error("Failed to read JSON file: {0}")]
    ReadError(#[from] io::Error),
}
