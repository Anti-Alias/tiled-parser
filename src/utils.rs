use crate::{Error, Result};

pub fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => return Err(Error::InvalidLayerError),
    }
}