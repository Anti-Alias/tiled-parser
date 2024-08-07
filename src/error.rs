use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use thiserror::Error;

/// Any error that can occur during parsing.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse file")]
    ParsingError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Failed to parse layer")]
    InvalidLayerError,
    #[error("Failed to decode tile layers")]
    DecodeLayerError,
    #[error("XML parsing failed")]
    XmlParsingError,
    #[error("JSON parsing failed")]
    JsonParsingError,
}

impl From<ParseBoolError> for Error {
    fn from(_value: ParseBoolError) -> Self {
        Self::ParsingError
    }
}

impl From<ParseIntError> for Error {
    fn from(_value: ParseIntError) -> Self {
        Self::ParsingError
    }
}

impl From<ParseFloatError> for Error {
    fn from(_value: ParseFloatError) -> Self {
        Self::ParsingError
    }
}

impl From<roxmltree::Error> for Error {
    fn from(_value: roxmltree::Error) -> Self {
        Self::XmlParsingError
    }
}

impl From<serde_json::Error> for Error {
    fn from(_value: serde_json::Error) -> Self {
        Self::JsonParsingError
    }
}

pub type Result<T> = std::result::Result<T, Error>;