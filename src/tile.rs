use roxmltree::Node;
use crate::{Image, Properties, Tileset, TilesetEntry, Result};

/// A tile belonging to a [`Tileset`].
#[derive(Clone, Debug)]
pub struct Tile<'a> {
    id: u32,
    tileset: &'a Tileset,
    data: &'a TileData,
}

impl<'a> Tile<'a> {
    pub(crate) fn new(id: u32, tileset: &'a Tileset, data: &'a TileData) -> Self {
        Self { id, tileset, data }
    }
    pub fn id(&self) -> u32 { self.id }
    pub fn typ(&self) -> &str { &self.data.typ }
    pub fn properties(&self) -> &Properties { &self.data.properties }
    pub fn image(&self) -> Option<&Image> { self.data.image.as_ref() }
    pub fn x(&self) -> Option<u32> { self.data.x }
    pub fn y(&self) -> Option<u32> { self.data.y }
    pub fn width(&self) -> Option<u32> { self.data.width }
    pub fn height(&self) -> Option<u32> { self.data.height }
    pub fn animation(&self) -> Option<&'a Animation> { self.data.animation.as_ref() }
    pub fn tileset(&self) -> &'a Tileset { self.tileset }

    /// Region of an image this tile belongs to.
    /// None if the tileset it belongs to is a collection.
    pub fn region(&self) -> Option<TilesetRegion> {
        if self.tileset.image().is_none() { return None }
        let columns = self.tileset.columns();
        let tile_width = self.tileset.tile_width();
        let tile_height = self.tileset.tile_height();
        let tile_x = self.id % columns;
        let tile_y = self.id / columns;
        let x = tile_x * tile_width;
        let y = tile_y * tile_height;
        let width = x + tile_width;
        let height = y + tile_height;
        Some(TilesetRegion { x, y, width, height })
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct TileData {
    typ: String,
    properties: Properties,
    image: Option<Image>,
    animation: Option<Animation>,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
}

impl TileData {
    pub fn parse(tile_node: Node) -> Result<(u32, TileData)> {

        // Attributes
        let mut id = 0;
        let mut result = Self::default();
        for attr in tile_node.attributes() {
            match attr.name() {
                "id" => id = attr.value().parse()?,
                "x" => result.x = Some(attr.value().parse()?),
                "y" => result.y = Some(attr.value().parse()?),
                "width" => result.width = Some(attr.value().parse()?),
                "height" => result.height = Some(attr.value().parse()?),
                _ => {}
            }
        }

        // Children
        for child in tile_node.children() {
            match child.tag_name().name() {
                "properties" => result.properties = Properties::parse(child)?,
                "image" => result.image = Some(Image::parse(child)?),
                "animation" => result.animation = Some(Animation::parse(child)?),
                _ => {}
            }
        }

        Ok((id, result))
    }
}

/// The region (in pixels) of an image a tile resides in.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct TilesetRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Global id of a [`Tile`] within a map.
/// Includes both the tileset index and tile index for faster lookups.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub enum Gid {
    #[default]
    Null,
    Value {
        tileset_index: u16,
        tile_id: u32,
        flip: Flip,
    },
}

impl Gid {

    const FLIP_FLAGS: u32 = 0b11110000_00000000_00000000_00000000;

    /// Converts a tiled map file's gid to a [`Gid`].
    pub(crate) fn resolve(gid: u32, entries: &[TilesetEntry]) -> Self {
        if gid == 0 { return Gid::Null }
        let flip_bits = (gid & Self::FLIP_FLAGS) >> 28;
        let gid = gid & !Self::FLIP_FLAGS;
        for (tileset_index, tileset_entry) in entries.iter().enumerate().rev() {
            if gid >= tileset_entry.first_gid() {
                return Gid::Value {
                    tileset_index: tileset_index as u16,
                    tile_id: gid - tileset_entry.first_gid(),
                    flip: Flip(flip_bits as u8)
                }
            }
        }
        Self::Null
    }
}

/// Contains information about how a tile is flipped and rotated in a map.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Flip(pub u8);
impl Flip {
    
    pub const FLIPPED_HORIZONTALLY_FLAG: u8     = 0b00001000;
    pub const FLIPPED_VERTICALLY_FLAG: u8       = 0b00000100;
    pub const FLIPPED_DIAGONALLY_FLAG: u8       = 0b00000010;
    pub const ROTATED_HEXAGONAL_120_FLAG: u8    = 0b00000001;

    pub fn is_flipped_horizontal(self) -> bool {
        self.0 & Self::FLIPPED_HORIZONTALLY_FLAG != 0
    }

    pub fn is_flipped_vertical(self) -> bool {
        self.0 & Self::FLIPPED_VERTICALLY_FLAG != 0
    }

    pub fn is_flipped_diagonal(self) -> bool {
        self.0 & Self::FLIPPED_DIAGONALLY_FLAG != 0
    }

    pub fn is_rotated_hex_120(self) -> bool {
        self.0 & Self::ROTATED_HEXAGONAL_120_FLAG != 0
    }
}

/// Animation frames of a [`Tile`].
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Animation(Vec<Frame>);
impl Animation {
    
    pub fn frames(&self) -> &[Frame] { &self.0 }

    pub(crate) fn parse(animation_node: Node) -> Result<Self> {
        let mut frames = Vec::new();
        for frame_node in animation_node.children().filter(|node| node.tag_name().name() == "frame") {
            let mut frame = Frame::default();
            for attr in frame_node.attributes() {
                match attr.name() {
                    "tileid" => frame.tile_id = attr.value().parse()?,
                    "duration" => frame.duration = attr.value().parse()?,
                    _ => {}
                }
            }
            frames.push(frame);
        }
        Ok(Self(frames))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Frame {
    pub tile_id: u32,
    pub duration: u32,
}

impl std::fmt::Debug for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Flip {{ horizontal: {}, vertical: {}, diagonal: {}, rotated_hex_120: {} }}",
            self.is_flipped_horizontal(), self.is_flipped_vertical(), self.is_flipped_diagonal(), self.is_rotated_hex_120()
        )
    }
}