use std::num::{ParseIntError, ParseFloatError};
use std::str::ParseBoolError;
use base64::DecodeError;
use derive_more::*;

#[derive(Error, Display, From, Debug)]
pub enum Error {
    XmlError(roxmltree::Error),
    #[display(fmt="{_0}")]
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    ParseBoolError(ParseBoolError),
    Base64Error(DecodeError),
    ParseColorError,
    IOError(std::io::Error),
    #[display(fmt="Unexpected tag {_0}")]
    #[from(ignore)]
    #[error(ignore)]
    UnexpectedTagError(String),
    #[display(fmt="Unexpected value {_0}")]
    #[from(ignore)]
    #[error(ignore)]
    InvalidAttributeValue(String),
    #[display(fmt="Invalid node")]
    InvalidNode,
    #[display(fmt="Missing attribute {_0}")]
    #[from(ignore)]
    #[error(ignore)]
    MissingAttribute(&'static str),
    #[display(fmt="Missing layer data")]
    MissingLayerData,
    #[display(fmt="Missing child with tag name {_0}")]
    #[from(ignore)]
    #[error(ignore)]
    MissingChild(&'static str),
    #[display(fmt="Embedded images not supported")]
    EmbeddedImagesNotSupported,
    #[display(fmt="Unsupported encoding/compression")]
    UnsupportedEncodingAndCompression,
    #[display(fmt="Layer missing data")]
    LayerMissingData,
    #[display(fmt="Invalid GID")]
    InvalidGid,
    #[display(fmt="Referenced tile not found")]
    TileNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;