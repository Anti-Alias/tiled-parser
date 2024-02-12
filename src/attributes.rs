use roxmltree::Node;

use crate::{Error, Result};

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Orientation {
    #[default]
    Orthogonal,
    Isometric,
    Staggered,
}

impl Orientation {
    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "orthogonal" => Ok(Self::Orthogonal),
            "isometric" => Ok(Self::Isometric),
            "staggered" => Ok(Self::Staggered),
            _ => Err(Error::InvalidAttributeValue(value.into()))
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum ObjectAlignment {
    #[default]
    Unspecified,
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum FillMode {
    #[default]
    Stretch,
    PreserveAspectFit,
}

impl FillMode {
    pub fn parse(str: &str) -> Result<Self> {
        match str {
            "stretch" => Ok(Self::Stretch),
            "preserve-aspect-fit" => Ok(Self::PreserveAspectFit),
            _ => Err(Error::InvalidAttributeValue(str.into()))
        }
    }
}

impl ObjectAlignment {
    pub fn parse(str: &str) -> Result<Self> {
        match str {
            "unspecified" => Ok(Self::Unspecified),
            "topleft" => Ok(Self::TopLeft),
            "top" => Ok(Self::Top),
            "topright" => Ok(Self::TopRight),
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            "bottomleft" => Ok(Self::BottomLeft),
            "bottom" => Ok(Self::Bottom),
            "bottomright" => Ok(Self::BottomRight),
            _ => Err(Error::InvalidAttributeValue(str.into()))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum TileRenderSize {
    #[default]
    Tile,
    Grid,
}

impl TileRenderSize {
    pub fn parse(str: &str) -> Result<Self> {
        match str {
            "tile" => Ok(Self::Tile),
            "grid" => Ok(Self::Grid),
            _ => Err(Error::InvalidAttributeValue(str.into()))
        }
    }
}

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

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Grid {
    pub orientation: Orientation,
    pub width: u32,
    pub height: u32,
}

impl Grid {
    pub(crate) fn parse(node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in node.attributes() {
            match attr.name() {
                "orientation" => result.orientation = Orientation::from_str(attr.value())?,
                "width" => result.width = attr.value().parse()?,
                "height" => result.height = attr.value().parse()?,
                _ => {}
            }
        }
        Ok(result)
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
}

impl Color {

    pub(crate) fn from_hex_argb(s: &str) -> Result<Self> {
        let s =
            if s.starts_with('#') { &s[1..] }
            else { s };
        match s.len() {
            6 => {
                let rgb = u32::from_str_radix(s, 16).map_err(|_| Error::ParseColorError)?;
                Ok(Self::from_rgb(rgb))
            },
            8 => {
                let argb = u32::from_str_radix(s, 16).map_err(|_| Error::ParseColorError)?;
                Ok(Self::from_argb(argb))
            },
            _ => Err(Error::ParseColorError),
        }
    }

    pub fn from_argb(value: u32) -> Self {
        let a = (value >> 24) & 0xFF;
        let r = (value >> 16) & 0xFF;
        let g = (value >> 8) & 0xFF;
        let b = value & 0xFF;
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn from_rgb(value: u32) -> Self {
        let r = (value >> 16) & 0xFF;
        let g = (value >> 8) & 0xFF;
        let b = value & 0xFF;
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
}