use std::str::FromStr;
use roxmltree::Node;
use crate::{Error, Result};

/// Orientation of the map.
/// Either Orthogonal, Isometric, Staggered or Hexagonal.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Orientation {
    #[default]
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

impl Orientation {
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "orthogonal" => Ok(Self::Orthogonal),
            "isometric" => Ok(Self::Isometric),
            "staggered" => Ok(Self::Staggered),
            "hexagonal" => Ok(Self::Hexagonal),
            _ => Err(Error::ParsingError),
        }
    }
}

/// Offset applied to a tile when drawn from a tileset.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct TileOffset { pub x: i32, pub y: i32 }
impl TileOffset {
    pub(crate) fn parse(node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in node.attributes() {
            match attr.name() {
                "x" => result.x = attr.value().parse()?,
                "y" => result.y = attr.value().parse()?,
                _ => {}
            }
        }
        Ok(result)
    }
}

/// An RGBA color.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self { Self::WHITE }
}

impl Color {

    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    fn from_argb(value: u32) -> Self {
        let a = (value >> 24) & 0xFF;
        let r = (value >> 16) & 0xFF;
        let g = (value >> 8) & 0xFF;
        let b = value & 0xFF;
        Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: a as u8,
        }
    }

    fn from_rgb(value: u32) -> Self {
        let r = (value >> 16) & 0xFF;
        let g = (value >> 8) & 0xFF;
        let b = value & 0xFF;
        Self {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        }
    }
}

impl FromStr for Color {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s =
            if s.starts_with('#') { &s[1..] }
            else { s };
        match s.len() {
            6 => {
                let rgb = u32::from_str_radix(s, 16).map_err(|_| Error::ParsingError)?;
                Ok(Self::from_rgb(rgb))
            },
            8 => {
                let argb = u32::from_str_radix(s, 16).map_err(|_| Error::ParsingError)?;
                Ok(Self::from_argb(argb))
            },
            _ => return Err(Error::ParsingError),
        }
    }
}