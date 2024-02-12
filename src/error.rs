use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse tmx file")]
    ParsingError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Failed to parse layer")]
    InvalidLayerError,
    #[error("Failed to decode tile layers")]
    DecodeLayerError,
    #[error("XML parsing failed")]
    XmlParsingError,
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

pub type Result<T> = std::result::Result<T, Error>;