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
    pub fn tileset(&self) -> &'a Tileset { self.tileset }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct TileData {
    typ: String,
    properties: Properties,
    image: Option<Image>,
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
                _ => {}
            }
        }

        Ok((id, result))
    }
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

    const FLIP_BITS: u32 = 0b11110000_00000000_00000000_00000000;

    /// Converts a tiled map file's gid to a [`Gid`].
    pub(crate) fn resolve(gid: u32, entries: &[TilesetEntry]) -> Self {
        let flip_bits = (gid & Self::FLIP_BITS) >> 28;
        let gid = gid & !Self::FLIP_BITS;
        for (tileset_index, tileset_entry) in entries.iter().enumerate() {
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

/// Contains information about how a tile is flipped in the map.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Flip(pub u8);
impl Flip {
    
    pub const FLIPPED_HORIZONTALLY_FLAG: u8     = 0b00001000;
    pub const FLIPPED_VERTICALLY_FLAG: u8       = 0b00000100;
    pub const FLIPPED_DIAGONALLY_FLAG: u8       = 0b00000010;
    pub const ROTATED_HEXAGONAL_120_FLAG: u8    = 0b00000001;

    pub fn flipped_horizontal(self) -> bool {
        self.0 & Self::FLIPPED_HORIZONTALLY_FLAG != 0
    }

    pub fn flipped_vertical(self) -> bool {
        self.0 & Self::FLIPPED_VERTICALLY_FLAG != 0
    }

    pub fn flipped_diagonal(self) -> bool {
        self.0 & Self::FLIPPED_DIAGONALLY_FLAG != 0
    }

    pub fn rotated_hex_120(self) -> bool {
        self.0 & Self::ROTATED_HEXAGONAL_120_FLAG != 0
    }
}

impl std::fmt::Debug for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Flip {{ horizontal: {}, vertical: {}, diagonal: {}, rotated_hex_120: {} }}",
            self.flipped_horizontal(), self.flipped_vertical(), self.flipped_diagonal(), self.rotated_hex_120()
        )
    }
}